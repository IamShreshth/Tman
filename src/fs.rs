use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

pub fn list_dir(dir: &Path, hidden: bool) -> Vec<Entry> {
    let mut v: Vec<Entry> = match fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                if !hidden && name.starts_with('.') {
                    return None;
                }
                let path = e.path();
                let is_dir = path.is_dir();
                Some(Entry { name, path, is_dir })
            })
            .collect(),
        Err(_) => vec![],
    };
    v.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    v
}

/// Breadth-first recursive listing (shallow results first), bounded.
pub fn walk(root: &Path, hidden: bool, max_depth: usize, cap: usize) -> Vec<Entry> {
    const SKIP: [&str; 5] = ["node_modules", "target", "__pycache__", ".venv", ".git"];
    let mut out = vec![];
    let mut q = VecDeque::from([(root.to_path_buf(), 0usize)]);
    while let Some((dir, depth)) = q.pop_front() {
        for e in list_dir(&dir, hidden) {
            if out.len() >= cap {
                return out;
            }
            let rel = e.path.strip_prefix(root).unwrap_or(&e.path).to_string_lossy().into_owned();
            let is_link = fs::symlink_metadata(&e.path).map(|m| m.file_type().is_symlink()).unwrap_or(true);
            if e.is_dir && depth < max_depth && !is_link && !SKIP.contains(&e.name.as_str()) {
                q.push_back((e.path.clone(), depth + 1));
            }
            out.push(Entry { name: rel, ..e });
        }
    }
    out
}

pub fn head(path: &Path, max_lines: usize) -> Vec<String> {
    use std::io::Read;
    let mut buf = vec![0u8; 16 * 1024];
    let n = match fs::File::open(path).and_then(|mut f| f.read(&mut buf)) {
        Ok(n) => n,
        Err(e) => return vec![format!("(cannot read: {e})")],
    };
    buf.truncate(n);
    if buf.contains(&0) {
        return vec!["(binary file)".into()];
    }
    String::from_utf8_lossy(&buf)
        .lines()
        .take(max_lines)
        .map(|l| l.replace('\t', "    "))
        .collect()
}

pub fn shell_quote(s: &str) -> String {
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || "/._-+=:@,~".contains(c)) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

pub fn display_path(p: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME") {
        if let Ok(rest) = p.strip_prefix(PathBuf::from(home)) {
            return if rest.as_os_str().is_empty() { "~".into() } else { format!("~/{}", rest.display()) };
        }
    }
    p.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quoting() {
        assert_eq!(shell_quote("src/a.rs"), "src/a.rs");
        assert_eq!(shell_quote("my file"), "'my file'");
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
    }
}
