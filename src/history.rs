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
    // zsh extended format: ": 1700000000:0;command"
    if line.starts_with(": ") {
        if let Some((_, c)) = line.split_once(';') {
            return c.trim();
        }
    }
    line.trim()
}

#[cfg(test)]
mod tests {
    #[test]
    fn zsh_extended() {
        assert_eq!(super::parse_line(": 1700000000:0;git status"), "git status");
        assert_eq!(super::parse_line("  ls -la"), "ls -la");
    }
}
