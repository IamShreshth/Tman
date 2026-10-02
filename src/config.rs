use serde::Deserialize;
use std::path::PathBuf;

pub const DEFAULT_TOML: &str = r##"[ui]
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
    pub ui: Ui,
    pub keybindings: Keybindings,
    pub shell: Shell,
    pub safety: Safety,
    pub bookmarks: Vec<String>,
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
            Ok(s) => toml::from_str(&s).unwrap_or_else(|e| {
                eprintln!("tman: ignoring invalid config ({e})");
                Config::default()
            }),
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
        self.bookmarks.iter().any(|b| b == path)
    }

    pub fn toggle_bookmark(&mut self, path: String) -> bool {
        if self.is_bookmarked(&path) {
            self.remove_bookmark(&path);
            false
        } else {
            self.add_bookmark(path);
            true
        }
    }

    pub fn add_bookmark(&mut self, path: String) {
        if !self.bookmarks.contains(&path) {
            self.bookmarks.push(path);
            self.save_bookmarks();
        }
    }

    pub fn remove_bookmark(&mut self, path: &str) {
        self.bookmarks.retain(|b| b != path);
        self.save_bookmarks();
    }

    pub fn save_bookmarks(&self) {
        let p = path();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let current = std::fs::read_to_string(&p).unwrap_or_else(|_| DEFAULT_TOML.to_string());
        let mut new_lines = Vec::new();
        let mut found = false;
        let mut in_array = false;
        let formatted_b = format!("bookmarks = {:?}", self.bookmarks);
        for line in current.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("bookmarks =") || trimmed.starts_with("bookmarks=") {
                new_lines.push(formatted_b.clone());
                found = true;
                if trimmed.contains('[') && !trimmed.contains(']') {
                    in_array = true;
                }
            } else if in_array {
                if trimmed.contains(']') {
                    in_array = false;
                }
            } else {
                new_lines.push(line.to_string());
            }
        }
        if !found {
            new_lines.push(String::new());
            new_lines.push(formatted_b);
        }
        let _ = std::fs::write(&p, new_lines.join("\n") + "\n");
    }
}
