use serde::Deserialize;
use std::path::PathBuf;

pub const DEFAULT_TOML: &str = r##"bookmarks = []

[ui]
animations = true     # slide-in context panel; set false to disable all motion
preview = true        # context/preview panel
show_hidden = false
accent = "cyan"       # cyan | blue | green | magenta | yellow | red | orange | purple | pink | peach | teal | white | #hex

[keybindings]
open = "ctrl-shift-t"   # read by `tman init <shell>`; examples: ctrl-g, alt-n

[shell]
editor = ""           # empty = $VISUAL / $EDITOR / vi

[safety]
confirm_destructive = true
"##;

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct Ui { pub animations: bool, pub preview: bool, pub show_hidden: bool, pub accent: String }
#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct Keybindings { pub open: String }
#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct Shell { pub editor: String }
#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct Safety { pub confirm_destructive: bool }
#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub bookmarks: Vec<String>,
    pub ui: Ui,
    pub keybindings: Keybindings,
    pub shell: Shell,
    pub safety: Safety,
}

impl Default for Ui { fn default() -> Self { Ui { animations: true, preview: true, show_hidden: false, accent: "cyan".into() } } }
impl Default for Keybindings { fn default() -> Self { Keybindings { open: "ctrl-shift-t".into() } } }
impl Default for Safety { fn default() -> Self { Safety { confirm_destructive: true } } }

pub fn path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("tman").join("config.toml")
}

impl Config {
    pub fn load() -> Config {
        match std::fs::read_to_string(path()) {
            Ok(s) => {
                let mut cfg: Config = toml::from_str(&s).unwrap_or_else(|e| {
                    eprintln!("tman: ignoring invalid config ({e})");
                    Config::default()
                });
                // Backward-compatibility: if bookmarks was placed after [safety],
                // serde assigned it to the safety table or omitted it from root.
                if cfg.bookmarks.is_empty() {
                    if let Ok(val) = toml::from_str::<toml::Value>(&s) {
                        if let Some(arr) = val.get("bookmarks")
                            .or_else(|| val.get("safety").and_then(|t| t.get("bookmarks")))
                            .and_then(|v| v.as_array())
                        {
                            cfg.bookmarks = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
                        }
                    }
                }
                cfg
            }
            Err(_) => Config::default(),
        }
    }
    pub fn editor(&self) -> String {
        if !self.shell.editor.is_empty() { return self.shell.editor.clone(); }
        std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")).unwrap_or_else(|_| "vi".into())
    }
    pub fn set_accent(&mut self, accent: &str) {
        self.ui.accent = accent.to_string();
        let p = path();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let current = std::fs::read_to_string(&p).unwrap_or_else(|_| DEFAULT_TOML.to_string());
        let mut new_lines = Vec::new();
        let mut found = false;
        for line in current.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("accent =") || trimmed.starts_with("accent=") {
                let comment = if let Some((_, c)) = line.split_once('#') {
                    format!(" #{}", c)
                } else {
                    String::new()
                };
                new_lines.push(format!("accent = \"{}\"{}", accent, comment));
                found = true;
            } else {
                new_lines.push(line.to_string());
            }
        }
        if !found {
            new_lines.push(format!("accent = \"{}\"", accent));
        }
        let _ = std::fs::write(&p, new_lines.join("\n") + "\n");
    }

    pub fn is_bookmarked(&self, path: &str) -> bool {
        let p_clean = path.trim_end_matches('/');
        self.bookmarks.iter().any(|b| b.trim_end_matches('/') == p_clean)
    }

    pub fn toggle_bookmark(&mut self, path: String) -> bool {
        let p_clean = path.trim_end_matches('/').to_string();
        if self.is_bookmarked(&p_clean) {
            self.remove_bookmark(&p_clean);
            false
        } else {
            self.add_bookmark(p_clean);
            true
        }
    }

    pub fn add_bookmark(&mut self, path: String) {
        let p_clean = path.trim_end_matches('/').to_string();
        if !self.is_bookmarked(&p_clean) {
            self.bookmarks.push(p_clean);
            self.save_bookmarks();
        }
    }

    pub fn remove_bookmark(&mut self, path: &str) {
        let p_clean = path.trim_end_matches('/');
        self.bookmarks.retain(|b| b.trim_end_matches('/') != p_clean);
        self.save_bookmarks();
    }

    pub fn save_bookmarks(&self) {
        let p = path();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let current = std::fs::read_to_string(&p).unwrap_or_else(|_| DEFAULT_TOML.to_string());
        let updated = format_bookmarks_toml(&current, &self.bookmarks);
        let _ = std::fs::write(&p, updated);
    }
}

pub fn format_bookmarks_toml(current: &str, bookmarks: &[String]) -> String {
    let mut clean_lines = Vec::new();
    let mut in_array = false;
    for line in current.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("bookmarks =") || trimmed.starts_with("bookmarks=") {
            if trimmed.contains('[') && !trimmed.contains(']') {
                in_array = true;
            }
            continue;
        }
        if in_array {
            if trimmed.contains(']') {
                in_array = false;
            }
            continue;
        }
        clean_lines.push(line);
    }

    let formatted_b = format!("bookmarks = {:?}", bookmarks);
    let mut final_lines = Vec::new();
    let mut inserted = false;

    for line in clean_lines {
        let trimmed = line.trim_start();
        if !inserted && trimmed.starts_with('[') {
            final_lines.push(formatted_b.clone());
            final_lines.push(String::new());
            inserted = true;
        }
        final_lines.push(line.to_string());
    }

    if !inserted {
        final_lines.push(formatted_b);
    }

    let joined = final_lines.join("\n").trim_start().to_string();
    joined + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_and_parse_bookmarks() {
        let initial = r#"[ui]
accent = "cyan"

[safety]
confirm_destructive = true
"#;
        let bookmarks = vec!["/Users/test/dir1".to_string(), "/Users/test/dir2".to_string()];
        let formatted = format_bookmarks_toml(initial, &bookmarks);

        let cfg: Config = toml::from_str(&formatted).unwrap();
        assert_eq!(cfg.bookmarks, bookmarks);
        assert_eq!(cfg.ui.accent, "cyan");
    }

    #[test]
    fn test_backward_compat_safety_bookmarks() {
        // Simulates old buggy format where bookmarks was written after [safety]
        let old_format = r#"[ui]
accent = "cyan"

[safety]
confirm_destructive = true

bookmarks = ["/Users/legacy/dir"]
"#;
        let mut cfg: Config = toml::from_str(old_format).unwrap();
        if cfg.bookmarks.is_empty() {
            let val: toml::Value = toml::from_str(old_format).unwrap();
            if let Some(arr) = val.get("safety").and_then(|t| t.get("bookmarks")).and_then(|v| v.as_array()) {
                cfg.bookmarks = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            }
        }
        assert_eq!(cfg.bookmarks, vec!["/Users/legacy/dir"]);

        // Resaving migrates it to the root
        let migrated = format_bookmarks_toml(old_format, &cfg.bookmarks);
        let reloaded: Config = toml::from_str(&migrated).unwrap();
        assert_eq!(reloaded.bookmarks, vec!["/Users/legacy/dir"]);
    }

    #[test]
    fn test_path_normalization() {
        let mut cfg = Config::default();
        cfg.add_bookmark("/Users/test/dir/".into());
        assert!(cfg.is_bookmarked("/Users/test/dir"));
        assert!(cfg.is_bookmarked("/Users/test/dir/"));
        assert_eq!(cfg.bookmarks, vec!["/Users/test/dir"]);

        cfg.toggle_bookmark("/Users/test/dir".into());
        assert!(!cfg.is_bookmarked("/Users/test/dir"));
        assert!(cfg.bookmarks.is_empty());
    }
}

