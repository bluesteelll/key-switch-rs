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
//
// The decision logic is a pure function so it can be unit-tested: cargo never
// runs tests inside a build script, so src/lib.rs includes this file as a
// `#[cfg(test)]` module and the tests below run with the library tests.

/// The `cargo:` directives for one build of this package.
///
/// `is_windows`, `target_env` and `profile` are the values of
/// `CARGO_CFG_WINDOWS` (set or not), `CARGO_CFG_TARGET_ENV` and `PROFILE`.
fn directives(is_windows: bool, target_env: &str, profile: &str) -> Vec<String> {
    let mut out = Vec::new();

    // `/MANIFEST:EMBED` and `/MANIFESTUAC` are MSVC linker switches. GNU ld
    // takes them for input file names and the link fails (KS-42), so other
    // Windows toolchains get a warning instead of the manifest.
    if is_windows && profile == "release" && target_env == "msvc" {
        // Explicitly request manifest embedding for the daemon binary, then
        // override the UAC fragment to `requireAdministrator`. rustc's
        // default link line doesn't always enable `/MANIFEST:EMBED`, so
        // /MANIFESTUAC alone would no-op without this companion flag.
        out.push("cargo:rustc-link-arg-bin=key-switch-rs=/MANIFEST:EMBED".to_owned());
        out.push(
            "cargo:rustc-link-arg-bin=key-switch-rs=/MANIFESTUAC:level='requireAdministrator' uiAccess='false'"
                .to_owned(),
        );
    } else if is_windows && profile == "release" {
        out.push(format!(
            "cargo:warning=no UAC manifest is embedded: /MANIFESTUAC needs the MSVC linker \
             (target env `{target_env}`), so key-switch-rs.exe will not request \
             administrator rights; start it elevated yourself"
        ));
    }

    out.push("cargo:rerun-if-changed=build.rs".to_owned());
    out
}

fn main() {
    let is_windows = std::env::var_os("CARGO_CFG_WINDOWS").is_some();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let profile = std::env::var("PROFILE").unwrap_or_default();

    for line in directives(is_windows, &target_env, &profile) {
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link_args(lines: &[String]) -> Vec<&String> {
        lines
            .iter()
            .filter(|l| l.starts_with("cargo:rustc-link-arg"))
            .collect()
    }

    fn warnings(lines: &[String]) -> Vec<&String> {
        lines
            .iter()
            .filter(|l| l.starts_with("cargo:warning="))
            .collect()
    }

    #[test]
    fn msvc_release_embeds_uac_manifest_into_daemon_only() {
        let lines = directives(true, "msvc", "release");
        assert_eq!(
            link_args(&lines),
            [
                "cargo:rustc-link-arg-bin=key-switch-rs=/MANIFEST:EMBED",
                "cargo:rustc-link-arg-bin=key-switch-rs=/MANIFESTUAC:level='requireAdministrator' uiAccess='false'",
            ]
        );
        assert!(warnings(&lines).is_empty());
    }

    #[test]
    fn gnu_release_gets_no_msvc_linker_flags_but_a_warning() {
        // KS-42: GNU ld treats /MANIFEST:EMBED and /MANIFESTUAC:... as input
        // files, so a windows-gnu release build failed at link time.
        let lines = directives(true, "gnu", "release");
        assert!(link_args(&lines).is_empty(), "{lines:?}");
        let warns = warnings(&lines);
        assert_eq!(warns.len(), 1, "{lines:?}");
        assert!(warns[0].contains("no UAC manifest"), "{}", warns[0]);
    }

    #[test]
    fn debug_builds_embed_nothing_and_do_not_warn() {
        for env in ["msvc", "gnu"] {
            let lines = directives(true, env, "debug");
            assert!(link_args(&lines).is_empty(), "{env}: {lines:?}");
            assert!(warnings(&lines).is_empty(), "{env}: {lines:?}");
        }
    }

    #[test]
    fn non_windows_targets_embed_nothing_and_do_not_warn() {
        let lines = directives(false, "gnu", "release");
        assert_eq!(lines, ["cargo:rerun-if-changed=build.rs"]);
    }

    /// Whether the `[[bin]]` table named `bin` in `manifest` sets
    /// `test = false`; `None` when there is no such table.
    ///
    /// Each line is compared without its comment and without whitespace, so
    /// `test=false`, `test = false` and `test = false # note` all count, and
    /// any line of the form `[...]` starts a new table, indented or not.
    /// Only the `[[bin]]` table form is read, not `bin = [{ ... }]`.
    fn bin_disables_tests(manifest: &str, bin: &str) -> Option<bool> {
        let mut tables: Vec<Vec<String>> = Vec::new();
        let mut in_bin = false;
        for line in manifest.lines() {
            let line: String = line
                .split('#')
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .collect();
            if line.starts_with('[') && line.ends_with(']') {
                in_bin = line == "[[bin]]";
                if in_bin {
                    tables.push(Vec::new());
                }
            } else if in_bin && let Some(table) = tables.last_mut() {
                table.push(line);
            }
        }
        let name = format!(r#"name="{bin}""#);
        let table = tables.iter().find(|table| table.contains(&name))?;
        Some(table.iter().any(|l| l == "test=false"))
    }

    #[test]
    fn bin_disables_tests_reads_the_named_bin_table() {
        let both = "[[bin]]\nname = \"swch\"\ntest = false\n\n\
                    [[bin]]\nname = \"key-switch-rs\"\npath = \"src/main.rs\"\n";
        assert_eq!(bin_disables_tests(both, "key-switch-rs"), Some(false));
        assert_eq!(bin_disables_tests(both, "swch"), Some(true));
        assert_eq!(bin_disables_tests("[lib]\ntest = false\n", "swch"), None);
        let enabled = "[[bin]]\nname = \"key-switch-rs\"\ntest = true\n";
        assert_eq!(bin_disables_tests(enabled, "key-switch-rs"), Some(false));
    }

    #[test]
    fn bin_disables_tests_ignores_whitespace_and_comments() {
        for manifest in [
            "[[bin]]\nname=\"key-switch-rs\"\ntest=false\n",
            "[[bin]]\nname = \"key-switch-rs\" # daemon\ntest = false # KS-38\n",
            "[[ bin ]]\n  name  =  \"key-switch-rs\"\n\ttest\t=\tfalse\n",
        ] {
            assert_eq!(
                bin_disables_tests(manifest, "key-switch-rs"),
                Some(true),
                "{manifest}"
            );
        }
    }

    #[test]
    fn bin_disables_tests_stops_at_an_indented_table_header() {
        // `test = false` here belongs to [lib], not to the binary.
        let manifest = "[[bin]]\nname = \"key-switch-rs\"\n  [lib]\n  test = false\n";
        assert_eq!(bin_disables_tests(manifest, "key-switch-rs"), Some(false));
    }

    #[test]
    fn daemon_bin_builds_no_test_harness() {
        // KS-38: the daemon's requireAdministrator manifest also lands in a
        // unit-test harness built from src/main.rs, which then fails with
        // "requires elevation" (os error 740) in a normal shell. Cargo.toml
        // must keep `test = false` on the key-switch-rs binary.
        let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
        assert_eq!(
            bin_disables_tests(manifest, "key-switch-rs"),
            Some(true),
            "Cargo.toml must declare [[bin]] name = \"key-switch-rs\" with test = false"
        );
    }
}
