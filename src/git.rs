use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitInfo {
    pub branch: String,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub ahead: usize,
    pub behind: usize,
    pub root: PathBuf,
}

fn git(cwd: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

fn parse_head(head: &str) -> (String, usize, usize) {
    let head = head.trim_start_matches("## ");
    let branch = if let Some(b) = head.strip_prefix("No commits yet on ") {
        b.to_string()
    } else if head.starts_with("HEAD (no branch)") {
        "detached".into()
    } else {
        head.split("...").next().unwrap_or(head).split(' ').next().unwrap_or(head).to_string()
    };
    let (mut ahead, mut behind) = (0, 0);
    if let Some( bracket ) = head.split('[').nth(1).and_then(|s| s.strip_suffix(']')) {
        for part in bracket.split(',') {
            let p = part.trim();
            if let Some(n) = p.strip_prefix("ahead ").and_then(|v| v.parse().ok()) { ahead = n; }
            if let Some(n) = p.strip_prefix("behind ").and_then(|v| v.parse().ok()) { behind = n; }
        }
    }
    (branch, ahead, behind)
}

pub fn info(cwd: &Path) -> Option<GitInfo> {
    let root = PathBuf::from(git(cwd, &["rev-parse", "--show-toplevel"])?.trim());
    let st = git(cwd, &["status", "--porcelain=v1", "-b"])?;
    let mut lines = st.lines();
    let (branch, ahead, behind) = parse_head(lines.next().unwrap_or(""));
    let (mut staged, mut modified, mut untracked) = (0, 0, 0);
    for l in lines {
        let b = l.as_bytes();
        if b.len() < 2 {
            continue;
        }
        if &b[..2] == b"??" {
            untracked += 1;
        } else {
            if b[0] != b' ' {
                staged += 1;
            }
            if b[1] != b' ' {
                modified += 1;
            }
        }
    }
    Some(GitInfo { branch, staged, modified, untracked, ahead, behind, root })
}

/// Stages an unstaged/untracked file, or unstages a purely staged file.
pub fn toggle_stage(cwd: &Path, path: &Path, status: &str) {
    let p = path.to_string_lossy();
    let is_unstaged = status == "??" || status.ends_with('M') || status.ends_with('D') || status.len() == 1 && !status.starts_with('A');
    if status == "??" || status.chars().nth(1).map(|c| c != ' ').unwrap_or(false) {
        let _ = git(cwd, &["add", "--", &p]);
    } else if !is_unstaged || !status.is_empty() {
        let _ = git(cwd, &["restore", "--staged", "--", &p])
            .or_else(|| git(cwd, &["reset", "HEAD", "--", &p]));
    }
}

/// (status code, absolute path)
pub fn changed_files(cwd: &Path) -> Vec<(String, PathBuf)> {
    let Some(root) = git(cwd, &["rev-parse", "--show-toplevel"]) else { return vec![] };
    let root = PathBuf::from(root.trim());
    git(cwd, &["status", "--porcelain=v1"])
        .unwrap_or_default()
        .lines()
        .filter(|l| l.len() > 3)
        .map(|l| {
            let p = l[3..].rsplit(" -> ").next().unwrap_or(&l[3..]).trim_matches('"');
            (l[..2].to_string(), root.join(p))
        })
        .collect()
}

pub fn diff_preview(cwd: &Path, path: &Path, max: usize) -> Vec<String> {
    let p = path.to_string_lossy();
    git(cwd, &["diff", "HEAD", "--", &p])
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.lines().take(max).map(String::from).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_branch_and_ahead_behind() {
        assert_eq!(parse_head("## main...origin/main [ahead 2, behind 1]"), ("main".into(), 2, 1));
        assert_eq!(parse_head("## feat/ui...origin/feat/ui [ahead 3]"), ("feat/ui".into(), 3, 0));
        assert_eq!(parse_head("## HEAD (no branch)"), ("detached".into(), 0, 0));
    }
}
