use std::cell::{Cell, RefCell};

use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{
            EVENT_OBJECT_REORDER, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND, GA_ROOT, GW_HWNDPREV,
            GWL_EXSTYLE, GetAncestor, GetDesktopWindow, GetWindow, GetWindowLongW, GetWindowRect,
            GetWindowThreadProcessId, HWND_TOPMOST, IsIconic, IsWindowVisible, OBJID_WINDOW,
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, SetWindowPos,
            WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WS_EX_TOPMOST,
        },
    },
};

thread_local! {
    static ISLAND: Cell<HWND> = const { Cell::new(HWND(std::ptr::null_mut())) };
    static HOOKS: RefCell<Vec<HWINEVENTHOOK>> = const { RefCell::new(Vec::new()) };
}

pub fn raise(hwnd: HWND) -> windows::core::Result<()> {
    // SAFETY: HWND da ilha. Preserva tamanho, posição e a janela que tem o teclado.
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER,
        )
    }
}

pub fn watch(hwnd: HWND) -> windows::core::Result<()> {
    // OUTOFCONTEXT entrega os eventos na thread que registra o hook, o loop da UI.
    ISLAND.set(hwnd);
    for event in [
        EVENT_SYSTEM_FOREGROUND,
        EVENT_OBJECT_SHOW,
        EVENT_OBJECT_REORDER,
    ] {
        // SAFETY: callback estático, sem DLL ou injeção em outro processo.
        let hook = unsafe {
            SetWinEventHook(
                event,
                event,
                None,
                Some(on_window_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };
        if hook.0.is_null() {
            let error = windows::core::Error::from_thread();
            unwatch();
            return Err(error);
        }
        HOOKS.with_borrow_mut(|hooks| hooks.push(hook));
    }
    Ok(())
}

pub fn unwatch() {
    ISLAND.set(HWND(std::ptr::null_mut()));
    HOOKS.with_borrow_mut(|hooks| {
        for hook in hooks.drain(..) {
            // SAFETY: cada hook é liberado na mesma thread que o registrou.
            let _ = unsafe { UnhookWinEvent(hook) };
        }
    });
}

unsafe extern "system" fn on_window_event(
    _: HWINEVENTHOOK,
    event: u32,
    event_window: HWND,
    object: i32,
    child: i32,
    _: u32,
    _: u32,
) {
    let island = ISLAND.get();
    if island.0.is_null() || event_window.0.is_null() || child != 0 {
        return;
    }
    // SAFETY: as consultas Win32 validam os handles. Os buffers RECT são locais.
    let covered = unsafe {
        if !IsWindowVisible(island).as_bool() || IsIconic(island).as_bool() {
            return;
        }

        // REORDER pode nomear a janela-pai. Não presumimos que seu HWND seja o
        // da janela que ficou acima. Ignoramos notificações internas de controles.
        if event_window != GetDesktopWindow()
            && (object != OBJID_WINDOW.0 || GetAncestor(event_window, GA_ROOT) != event_window)
        {
            return;
        }

        GetWindowLongW(island, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST.0 == 0
            || has_overlap_above(island, (event == EVENT_OBJECT_SHOW).then_some(event_window))
    };
    if covered {
        let _ = raise(island);
    }
}

fn has_overlap_above(island: HWND, only: Option<HWND>) -> bool {
    let mut rect = RECT::default();
    let mut current = island;
    let mut process = 0;
    // SAFETY: apenas consultas de posição, visibilidade e identidade da janela.
    unsafe {
        if GetWindowRect(island, &mut rect).is_err() {
            return false;
        }
        GetWindowThreadProcessId(island, Some(&mut process));
        for _ in 0..128 {
            let Ok(previous) = GetWindow(current, GW_HWNDPREV) else {
                break;
            };
            if previous.0.is_null() || previous == current {
                break;
            }
            current = previous;

            if only.is_some_and(|target| target != current)
                || !IsWindowVisible(current).as_bool()
                || IsIconic(current).as_bool()
            {
                continue;
            }
            let mut other_rect = RECT::default();
            let mut other_process = 0;
            GetWindowThreadProcessId(current, Some(&mut other_process));
            // Preservar o menu nativo e os pop-ups do próprio PixelPulse.
            if other_process != process
                && GetWindowRect(current, &mut other_rect).is_ok()
                && intersects(rect, other_rect)
            {
                return true;
            }
        }
    }
    false
}

fn intersects(a: RECT, b: RECT) -> bool {
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}
