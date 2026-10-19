//! The calls that put the daemon's hooks and its raw input sink into the
//! system and take them out again (roadmap item 2.4, ruling S1).

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    /// The Win32 calls that install or remove a global hook or a raw input
    /// sink. Assembled, so that this file does not match itself.
    const INSTALLERS: [&str; 5] = [
        concat!("SetWindows", "HookExW"),
        concat!("UnhookWindows", "HookEx"),
        concat!("SetWin", "EventHook"),
        concat!("UnhookWin", "Event"),
        concat!("RegisterRaw", "InputDevices"),
    ];

    /// The one place the calls may appear: the block of this module
    /// that opens with this line, right after `#[cfg(not(test))]`.
    const WIN32_BLOCK: &str = "mod win32 {";
    const INSTALLER_FILE: &str = "src/hook/installer.rs";

    /// Every `.rs` file under `dir`, recursively.
    fn rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("list a source folder") {
            let path = entry.expect("a folder entry").path();
            if path.is_dir() {
                rust_files(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }

    /// The index of the line that opens the tests module of a source file
    /// (`#[cfg(test)]` followed by `mod tests {`), or the line count.
    fn tests_start(lines: &[&str]) -> usize {
        lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {")
            .unwrap_or(lines.len())
    }

    /// The code of `line`: without its `//` comment and without the text of
    /// its string literals, so that a message or a doc that names a call
    /// is not taken for the call.
    fn code_of(line: &str) -> String {
        let mut code = String::new();
        let mut in_string = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if in_string {
                match c {
                    '\\' => {
                        chars.next();
                    }
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }
            match c {
                '"' => in_string = true,
                '/' if chars.peek() == Some(&'/') => break,
                _ => code.push(c),
            }
        }
        code
    }

    /// The numbers (from 1) of the lines of the block that opens at the
    /// first line equal to `opening` once trimmed, right after a
    /// `#[cfg(not(test))]` line, until its braces balance; `None` if there
    /// is no such block.
    fn cfg_not_test_block(
        lines: &[&str],
        opening: &str,
    ) -> Option<std::ops::RangeInclusive<usize>> {
        let start = lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(not(test))]" && pair[1].trim() == opening)?
            + 1;
        let mut depth = 0_i64;
        for (index, line) in lines.iter().enumerate().skip(start) {
            let code = code_of(line);
            depth += code.matches('{').count() as i64 - code.matches('}').count() as i64;
            if depth == 0 {
                return Some(start + 1..=index + 1);
            }
        }
        None
    }

    #[test]
    fn the_hook_installers_are_compiled_out_of_the_tests() {
        // Ruling S1 and S4 of roadmap item 2.4: every call that installs a
        // global hook or a raw input sink sits in the `win32` module of this
        // file, which `#[cfg(not(test))]` keeps out of the test build, so no
        // test can reach one, whatever a guard in front of it does. The
        // integration tests link the library as the daemon does, so none of
        // them may name a call, or the one public way to the hooks,
        // `core::app` (`App::run`).
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        rust_files(&root.join("src"), &mut files);
        files.sort();
        assert!(files.len() > 40, "the scan found the sources: {files:?}");

        let mut stray = Vec::new();
        let mut inside = Vec::new();
        let mut block = None;
        for file in &files {
            let text = std::fs::read_to_string(file).expect("read a source file");
            let relative = file
                .strip_prefix(root)
                .expect("a file under the manifest folder")
                .to_string_lossy()
                .replace('\\', "/");
            let lines: Vec<&str> = text.lines().collect();
            let allowed = if relative == INSTALLER_FILE {
                block = cfg_not_test_block(&lines, WIN32_BLOCK);
                block.clone()
            } else {
                None
            };
            for (index, line) in lines[..tests_start(&lines)].iter().enumerate() {
                let number = index + 1;
                let code = code_of(line);
                let calls: Vec<&str> = INSTALLERS
                    .into_iter()
                    .filter(|call| code.contains(call))
                    .collect();
                if allowed
                    .as_ref()
                    .is_some_and(|block| block.contains(&number))
                {
                    inside.extend(calls);
                } else if !calls.is_empty() {
                    stray.push(format!("{relative}:{number}: {}", line.trim()));
                }
            }
        }

        let mut integration = Vec::new();
        for entry in std::fs::read_dir(root.join("tests")).expect("list tests") {
            let path = entry.expect("a folder entry").path();
            let text = std::fs::read_to_string(&path).expect("read an integration test");
            for (index, line) in text.lines().enumerate() {
                let code = code_of(line);
                if INSTALLERS.iter().any(|call| code.contains(call)) || code.contains("core::app") {
                    integration.push(format!("{}:{}: {}", path.display(), index + 1, line.trim()));
                }
            }
        }

        let mut report = String::new();
        if block.is_none() {
            report +=
                &format!("{INSTALLER_FILE} has no `#[cfg(not(test))]` block `{WIN32_BLOCK}`\n");
        }
        for call in INSTALLERS {
            if block.is_some() && !inside.contains(&call) {
                report += &format!(
                    "{call} is not called in the win32 block, so the scan proves nothing about it\n"
                );
            }
        }
        if !stray.is_empty() {
            report += &format!(
                "{} installer calls a test build compiles:\n  {}\n",
                stray.len(),
                stray.join("\n  ")
            );
        }
        if !integration.is_empty() {
            report += &format!(
                "integration tests that reach a hook:\n  {}\n",
                integration.join("\n  ")
            );
        }
        assert!(report.is_empty(), "{report}");
    }
}
