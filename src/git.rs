use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitInfo {
    pub branch: String,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    #[allow(dead_code)]
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

pub fn info(cwd: &Path) -> Option<GitInfo> {
    let root = PathBuf::from(git(cwd, &["rev-parse", "--show-toplevel"])?.trim());
    let st = git(cwd, &["status", "--porcelain=v1", "-b"])?;
    let mut lines = st.lines();
    let head = lines.next().unwrap_or("").trim_start_matches("## ");
    let branch = if let Some(b) = head.strip_prefix("No commits yet on ") {
        b.to_string()
    } else if head.starts_with("HEAD (no branch)") {
        "detached".into()
    } else {
        head.split("...").next().unwrap_or(head).split(' ').next().unwrap_or(head).to_string()
    };
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
    Some(GitInfo { branch, staged, modified, untracked, root })
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
            (l[..2].trim().to_string(), root.join(p))
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
