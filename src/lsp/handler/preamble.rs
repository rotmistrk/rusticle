//! Shebang-based preamble discovery.
//!
//! Parses the shebang line to find the interpreter, then runs
//! `<interpreter> --lsp-preamble` to get command stubs.

use std::process::{Command, Stdio};
use std::time::Duration;

/// Extract interpreter name from a shebang line.
/// Handles `#!/usr/bin/env foo` and `#!/path/to/foo`.
pub fn parse_shebang(text: &str) -> Option<String> {
    let first_line = text.lines().next()?;
    if !first_line.starts_with("#!") {
        return None;
    }
    let rest = first_line[2..].trim();
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    // `#!/usr/bin/env interpreter` → take the second word
    // `#!/path/to/interpreter` → take basename of first word
    if parts[0].ends_with("/env") && parts.len() > 1 {
        Some(parts[1].to_string())
    } else {
        let path = parts[0];
        let basename = path.rsplit('/').next().unwrap_or(path);
        Some(basename.to_string())
    }
}

/// Run `interpreter --lsp-preamble` with stdin=/dev/null, 2s timeout.
/// Returns stdout on success, empty string on failure.
pub fn exec_lsp_preamble(interpreter: &str) -> String {
    let cmd = interpreter.to_string();
    let handle = std::thread::spawn(move || {
        Command::new(&cmd)
            .arg("--lsp-preamble")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()
    });
    let timeout = Duration::from_secs(2);
    let start = std::time::Instant::now();
    loop {
        if handle.is_finished() {
            return match handle.join() {
                Ok(Some(output)) if output.status.success() => {
                    String::from_utf8(output.stdout).unwrap_or_default()
                }
                _ => String::new(),
            };
        }
        if start.elapsed() > timeout {
            return String::new();
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shebang_env() {
        let text = "#!/usr/bin/env rusticle-tk\napp run\n";
        assert_eq!(parse_shebang(text), Some("rusticle-tk".into()));
    }

    #[test]
    fn shebang_absolute() {
        let text = "#!/usr/local/bin/rusticle-tk\napp run\n";
        assert_eq!(parse_shebang(text), Some("rusticle-tk".into()));
    }

    #[test]
    fn no_shebang() {
        assert_eq!(parse_shebang("set x 1\n"), None);
    }

    #[test]
    fn shebang_missing_interpreter() {
        let text = "#!/usr/bin/env\nset x 1\n";
        // env without argument — returns "env"
        assert_eq!(parse_shebang(text), Some("env".into()));
    }
}
