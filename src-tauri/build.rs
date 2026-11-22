fn main() {
    // Rebuild the Windows resource when the icon changes at the same path.
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-env-changed=PIXELPULSE_RELEASE_DATE");
    // Track HEAD and its branch ref so incremental builds identify the correct revision.
    for reference in [
        "HEAD".to_string(),
        git(&["symbolic-ref", "-q", "HEAD"]).unwrap_or_default(),
    ] {
        if !reference.is_empty()
            && let Some(path) = git(&["rev-parse", "--git-path", &reference])
        {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    if let Some(sha) = git(&["rev-parse", "--short=12", "HEAD"]) {
        println!("cargo:rustc-env=PIXELPULSE_REVISION={sha}");
    }
    tauri_build::build()
}

fn git(args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
