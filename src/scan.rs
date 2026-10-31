//! Test-only: the lexical helpers of the source scans, which read the
//! package's own files and pin where a call may appear: the hook installers
//! (`hook::installer`) and the clipboard calls (`platform::clipboard`).
//!
//! They read lines, not Rust: a line's code is the line without its `//`
//! comment and the text of its string literals, so that a message or a doc
//! that names a call is not taken for the call.

use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

/// Every `.rs` file under `dir`, recursively.
pub(crate) fn rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("list a source folder") {
        let path = entry.expect("a folder entry").path();
        if path.is_dir() {
            rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

/// The sources of the package under `root` that the scans read, sorted:
/// every `.rs` file under `src` and the build script, which Cargo runs at
/// every build and `lib.rs` compiles into the test harness.
pub(crate) fn scanned_sources(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![root.join("build.rs")];
    rust_files(&root.join("src"), &mut files);
    files.sort();
    files
}

/// The code of `line`: without its `//` comment and without the text of
/// its string literals, so that a message or a doc that names a call is not
/// taken for the call.
pub(crate) fn code_of(line: &str) -> String {
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

/// The numbers (from 1) of the lines of the block that opens at the first
/// line equal to `opening` once trimmed, right after a `#[cfg(not(test))]`
/// line, until its braces balance; `None` if there is no such block.
pub(crate) fn cfg_not_test_block(lines: &[&str], opening: &str) -> Option<RangeInclusive<usize>> {
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

/// The lines of `text` inside a fenced block of a doc comment, `///` or
/// `//!`, each with its number (from 1) and without the comment marker.
/// `cargo test` compiles such a block as a doctest, against the library
/// built without `cfg(test)`, so a call there reaches Windows.
pub(crate) fn doc_code_lines(text: &str) -> Vec<(usize, &str)> {
    let mut found = Vec::new();
    let mut in_fence = false;
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let Some(doc) = trimmed
            .strip_prefix("///")
            .or_else(|| trimmed.strip_prefix("//!"))
        else {
            in_fence = false;
            continue;
        };
        if doc.trim_start().starts_with("```") {
            in_fence = !in_fence;
        } else if in_fence {
            found.push((index + 1, doc));
        }
    }
    found
}

/// The lines of a file before its tests module, the `#[cfg(test)]` line
/// followed by `mod tests {`: all of them if it has none. They are what the
/// daemon compiles, the `#[cfg(test)]` items before the module aside.
pub(crate) fn production_lines<'a>(lines: &'a [&'a str]) -> &'a [&'a str] {
    let end = lines
        .windows(2)
        .position(|pair| pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {")
        .unwrap_or(lines.len());
    &lines[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_production_lines_end_at_the_tests_module() {
        let lines = ["fn a() {}", "#[cfg(test)]", "mod tests {", "}"];
        assert_eq!(production_lines(&lines), ["fn a() {}"]);
        assert_eq!(
            production_lines(&lines[..1]),
            ["fn a() {}"],
            "no tests module"
        );
    }

    #[test]
    fn the_code_of_a_line_leaves_out_its_comment_and_its_strings() {
        assert_eq!(code_of(r#"call("a // b"); // note"#), "call(); ");
        assert_eq!(code_of(r#"x("\"q\"") + y"#), "x() + y");
    }

    #[test]
    fn a_doc_fence_is_read_and_its_markers_left_out() {
        let text = [
            "/// Text before.",
            "/// ```",
            "/// let a = 1;",
            "/// ```",
            "/// After.",
            "//! ```rust",
            "//! inner();",
            "fn f() {}",
            "/// let not_in_a_fence = 2;",
        ]
        .join("\n");
        assert_eq!(
            doc_code_lines(&text),
            [(3, " let a = 1;"), (7, " inner();")]
        );
    }
}
