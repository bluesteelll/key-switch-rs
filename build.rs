// Embed a `requireAdministrator` UAC manifest into ONLY the daemon binary
// `key-switch-rs.exe`. The CLI `swch.exe` stays a normal-user process —
// it talks to the daemon via a named pipe; no elevation needed.
//
// Approach: rather than pass an external `.manifest` file (which conflicts
// with rustc's default auto-generated manifest), we override only the UAC
// fragment via the MSVC linker's `/MANIFESTUAC` switch. The default
// manifest stays in place for everything else (compatibility shims,
// dpiAware, etc. — defaults are sane). Scoping is per-binary via
// `cargo:rustc-link-arg-bin=<name>=<arg>`.

fn main() {
    let is_windows = std::env::var_os("CARGO_CFG_WINDOWS").is_some();
    let is_release = std::env::var("PROFILE").as_deref() == Ok("release");

    if is_windows && is_release {
        // Explicitly request manifest embedding for the daemon binary, then
        // override the UAC fragment to `requireAdministrator`. rustc's
        // default link line doesn't always enable `/MANIFEST:EMBED`, so
        // /MANIFESTUAC alone would no-op without this companion flag.
        println!("cargo:rustc-link-arg-bin=key-switch-rs=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=key-switch-rs=/MANIFESTUAC:level='requireAdministrator' uiAccess='false'"
        );
    }

    println!("cargo:rerun-if-changed=build.rs");
}
