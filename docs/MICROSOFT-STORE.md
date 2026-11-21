# Publish PixelPulse on the Microsoft Store

Use the **MSIX** submission path in Partner Center for the Windows 11 x64 desktop app. Windows manages installation and updates. On a system using the default app volume, the package usually lives under `C:\Program Files\WindowsApps`. Query the actual location with `Get-AppxPackage -Name '<identity name>' | Select-Object InstallLocation`. User preferences are stored separately in `%USERPROFILE%\.pixelpulse\settings.json` and may survive uninstall.

## 1. Prepare the publisher and product

1. Sign in to [Partner Center](https://partner.microsoft.com/dashboard) with the individual developer account enrolled in the Windows program. Finish identity verification and agreements, confirm the public publisher name and verify the account contact details. Set up tax and payout profiles if required for the distribution you choose.
2. In **Apps and games**, create a product using **MSIX or PWA** and reserve **PixelPulse**. If the reserved name differs, align the manifest, app branding and listing before submission.
3. Open **Product management → Product identity**. Copy **Package/Identity/Name**, **Package/Identity/Publisher** and **Package/Properties/PublisherDisplayName** exactly. Keep these stable across updates. The Tauri identifier serves a separate purpose.
4. Prepare a tested source revision, the listing copy, real screenshots, a public support contact and a privacy notice URL. Plan to make the corresponding GPL-3.0 source and build instructions accessible when the binary becomes available.

## 2. Install the SDK tools

Run in PowerShell on Windows x64. The Windows SDK provides MakeAppx, SignTool and the Windows App Certification Kit (WACK). In the SDK installer, enable the desktop tooling, signing tools and certification kit if offered as separate components.

```powershell
winget search --id Microsoft.WindowsSDK.10.0.28000 --exact --source winget
winget install --id Microsoft.WindowsSDK.10.0.28000 --exact --source winget
```

With the SDK installed, locate a single x64 tool directory containing both programs:

```powershell
$sdkRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
$sdkBin = Get-ChildItem -LiteralPath $sdkRoot -Directory |
    Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
    Sort-Object { [version]$_.Name } -Descending |
    ForEach-Object { Join-Path $_.FullName 'x64' } |
    Where-Object {
        (Test-Path (Join-Path $_ 'MakeAppx.exe')) -and
        (Test-Path (Join-Path $_ 'SignTool.exe'))
    } |
    Select-Object -First 1

if (-not $sdkBin) { throw 'Install the Windows SDK x64 packaging and signing tools.' }
$makeAppx = Join-Path $sdkBin 'MakeAppx.exe'
$signTool = Join-Path $sdkBin 'SignTool.exe'
& $makeAppx /?
& $signTool /?
```

See [Windows SDK installation](https://learn.microsoft.com/en-us/windows-hardware/drivers/install-the-wdk-using-winget#windows-sdk), [MakeAppx usage](https://learn.microsoft.com/en-us/windows/msix/package/create-app-package-with-makeappx-tool) and [SignTool usage](https://learn.microsoft.com/en-us/windows/msix/package/sign-app-package-using-signtool).

## 3. Build and validate the MSIX

Use a Windows x64 machine with Node.js, Rust MSVC and the Visual Studio C++ build tools. Build from the exact source revision intended for public distribution. Set the four-part Store version to match the release, for example `1.0.0.0` for app version `1.0.0`. Increment the accepted package version for updates and keep its last component at `0`.

```powershell
npm ci
npm run tauri build -- --no-bundle

.\src-tauri\packaging\build-msix.ps1 `
  -IdentityName '<Package/Identity/Name>' `
  -Publisher '<Package/Identity/Publisher>' `
  -PublisherDisplayName '<Package/Properties/PublisherDisplayName>' `
  -Version '1.0.0.0' `
  -MakeAppxPath $makeAppx
```

The package is under `src-tauri\target\msix`. Use the exact path printed by the script. Record the source revision and package hash, then unpack into a fresh temporary directory to review `AppxManifest.xml`, the executable, license and logos.

```powershell
$package = (Resolve-Path '<path printed by the build script>').Path
Get-FileHash -Algorithm SHA256 -LiteralPath $package
git rev-parse HEAD
$inspection = Join-Path ([IO.Path]::GetTempPath()) ("PixelPulse-" + [guid]::NewGuid())
& $makeAppx unpack /p $package /d $inspection
Get-Content (Join-Path $inspection 'AppxManifest.xml')
```

Review the identity against Partner Center, architecture `x64`, Windows 11 minimum build, app entry point, `runFullTrust` and the disabled-by-default startup task. Keep the app and task IDs stable. Verify the actual supported Windows and WebView2 versions in a clean environment.

For local installation, build a separate package with identity `PixelPulse.LocalTest` and publisher `CN=PixelPulse Local Test`. Sign it with a development certificate whose subject exactly matches that publisher. In a dedicated test VM, import the public certificate into Local Computer → Trusted People only after checking its fingerprint. Keep the signing key and PFX outside the repository.

```powershell
$testPackage = (Resolve-Path '<local-test MSIX path>').Path
& $signTool sign /fd SHA256 /f '<development PFX path>' $testPackage
if ($LASTEXITCODE -ne 0) { throw 'Local MSIX signing failed.' }
& $signTool verify /pa /v $testPackage
if ($LASTEXITCODE -ne 0) { throw 'Local MSIX signature verification failed.' }
Add-AppxPackage -Path $testPackage
Get-AppxPackage -Name 'PixelPulse.LocalTest' | Select-Object Version, InstallLocation
```

Run WACK against the installed app or package and keep its report. Test clean install, launch from Start, upgrade with a higher version and the same test identity, and uninstall. Include the following behavioral checks:

- Collapsed and expanded island, pin/unpin with ESC, display scaling, context menu and complete process exit.
- Diagnostics, optional volumes, language switching, preference persistence, and meaningful display when GPU sensors are unavailable.
- Startup off on a clean profile. Enable it, sign out and back in, verify launch, then disable and repeat. Turn off startup in a standalone copy before testing packaged startup.
- Release CPU and memory, WebView2 availability on the supported OS, and no duplicate island after sign-in.

Upload the package built with the real identity. Microsoft signs the distribution MSIX after certification. The development certificate is for local test installation. A standalone EXE has its own signing and distribution requirements. [Store signing FAQ](https://learn.microsoft.com/en-us/windows/apps/publish/faq/get-started-with-the-microsoft-store) · [Store readiness checklist](https://learn.microsoft.com/en-us/windows/apps/publish/store-readiness-checklist)

## 4. Create the submission in Partner Center

Open the reserved product and choose **Start your submission**. Complete the sections shown in your account and save each page before moving on.

| Section | What to enter and verify |
| --- | --- |
| **Pricing and availability** | Start with **Free** if that matches your release plan. Select supported markets and audience. For a coordinated launch, choose a publication hold or schedule when available, and review any market-specific requirements. |
| **Properties** | Choose the utilities/tools category offered by the form. Review publisher, copyright, website and licensing entries. Provide working HTTPS support and privacy URLs where requested. Describe actual features and dependencies only. |
| **Age ratings** | Complete the IARC questionnaire using the submitted build. Answer the questions about content, purchases, interaction, network use and data accurately. |
| **Packages** | Upload the real-identity MSIX. Wait for package validation and inspect architecture, version, supported device families and detected capabilities. |
| **Store listings** | Enter English (United States) first. Add Brazilian Portuguese and Spanish listings after proofreading if you want localized Store pages. App language support and Store listing language are separate. Prepare a description, feature list, release notes, keywords and screenshots as requested. |
| **Submission options** | Enter the test notes below and restricted capability justification if prompted. Review visibility, release timing and contact details. |

### English (United States) listing copy

| Field | Suggested copy |
| --- | --- |
| App name | PixelPulse |
| Short description | A lightweight pixel-art system monitor with a floating island for Windows. |
| Description | Keep your system metrics in sight with PixelPulse. The compact island shows the highest reading at a glance and expands to show CPU, memory, graphics and storage. Open System Diagnostics to inspect your PC, choose extra drives, change the language or opt in to launch at sign-in. Hardware readings and preferences stay on your PC. GPU metrics depend on driver and hardware support. |
| Features | Floating pixel-art island. CPU, memory, GPU and drive metrics. System diagnostics. Optional additional drives and startup. English (US), Brazilian Portuguese and Spanish interface. |
| Keywords | System monitor, CPU, memory, GPU, disk usage, pixel art |
| What's new | First public release. Includes a floating system monitor, local diagnostics, selectable drives, three interface languages and optional Windows startup. |
| Support and privacy | Enter the actual public support and privacy URLs. Check both while signed out. Use a support contact you monitor. |

Prepare screenshots from the exact release build: collapsed island, expanded metrics, Diagnostics, Devices and preferences. Use the Store's current size and format constraints. Keep text legible, crop out unrelated apps and remove account names, identifying paths and other personal information. Use the same project branding in the app, listing icon and screenshots.

### Notes for certification

> PixelPulse is a Windows desktop system monitor. No account is required. On launch, the island is centered at the top of the primary monitor. Hover to expand, click to pin and press ESC to unpin. Right-click the island and choose System Diagnostics to inspect the PC. In Devices, select another local drive if available. Startup is off by default and can be enabled by the user. Language offers English (US), Brazilian Portuguese and Spanish. Right-click and choose Exit PixelPulse to close the app. GPU sensors and drive-type identification may vary with hardware and drivers.

If the form requests a restricted capability explanation, explain that `runFullTrust` is needed for the packaged Win32 Tauri process to read local Windows metrics and manage its floating window using standard Windows APIs. The app runs at the user's privilege level. Explain the opt-in `StartupTask` separately.

Describe the local readings and settings accurately in the privacy notice, including how to remove the local settings file. Review licensing for the distributed binary and assets, including GPL-3.0 corresponding source and the embedded font's OFL notice. Check the current Store terms against the license before submitting.

## 5. Submit and release

1. Preview the listing in each supported language. Check links, screenshots, regions, price, package identity, validation warnings and the chosen publication timing. Archive the exact MSIX, its SHA-256, source revision, WACK report and submission text.
2. Select **Submit for certification**. Monitor certification in Partner Center. If Microsoft requests a change, address the specific finding, rebuild and retest. Use a higher package version when replacing a package as required by the portal.
3. When certified, release using the selected timing. Open the listing while signed out and verify localized pages, support links and accessibility. On a clean Windows account, install from the Store and smoke-test launch, settings, startup and exit.
4. Keep the listing and corresponding source link accurate for the distributed version. For updates, retain the same product and package identity, increase the version, and repeat package and install validation.

[Create a Store submission](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/create-app-submission?pivots=store-installer-msix) · [MSIX package requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/package-version-numbering) · [GPL-3.0 license](https://www.gnu.org/licenses/gpl-3.0.html)
