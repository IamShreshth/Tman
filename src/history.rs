use std::collections::HashSet;
use std::{env, fs};

/// Loads the user's *real* shell history, most recent first, deduplicated.
/// Prefers TMAN_HISTORY_FILE (written by the shell widget from the live session),
/// then $HISTFILE, then ~/.zsh_history / ~/.bash_history.
pub fn load() -> Vec<String> {
    let home = env::var("HOME").unwrap_or_default();
    let candidates = [
        env::var("TMAN_HISTORY_FILE").ok(),
        env::var("HISTFILE").ok(),
        Some(format!("{home}/.zsh_history")),
        Some(format!("{home}/.bash_history")),
    ];
    let bytes = candidates.into_iter().flatten().find_map(|p| fs::read(p).ok().filter(|b| !b.is_empty())).unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes);
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for line in text.lines().rev() {
        let cmd = parse_line(line);
        if !cmd.is_empty() && seen.insert(cmd.to_string()) {
            out.push(cmd.to_string());
            if out.len() >= 20_000 {
                break;
            }
        }
    }
    out
}

fn parse_line(line: &str) -> &str {
    let trimmed = line.trim();
    // zsh extended format: ": 1700000000:0;command"
    if trimmed.starts_with(": ") {
        if let Some((_, c)) = trimmed.split_once(';') {
            return c.trim();
        }
    }
    // bash timestamp format with HISTTIMEFORMAT: "#1700000000"
    if trimmed.starts_with('#') {
        let rest = &trimmed[1..];
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            return "";
        }
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zsh_extended() {
        assert_eq!(parse_line(": 1700000000:0;git status"), "git status");
        assert_eq!(parse_line("  ls -la"), "ls -la");
    }

    #[test]
    fn bash_timestamps() {
        assert_eq!(parse_line("#1700000000"), "");
        assert_eq!(parse_line("  #1680000000  "), "");
        assert_eq!(parse_line("# regular comment"), "# regular comment");
        assert_eq!(parse_line("cargo test #1"), "cargo test #1");
    }
}
