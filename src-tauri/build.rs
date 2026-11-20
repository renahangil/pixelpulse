fn main() {
    // Rebuild the Windows resource when the icon changes at the same path.
    println!("cargo:rerun-if-changed=icons/icon.ico");
    tauri_build::build()
}
