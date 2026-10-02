/// Returns a human-readable reason if the command looks destructive.
pub fn classify(cmd: &str) -> Option<&'static str> {
    for seg in cmd.split(|c| c == ';' || c == '|' || c == '&' || c == '\n') {
        let w: Vec<&str> = seg.split_whitespace().collect();
        let mut i = 0;
        while i < w.len()
            && (matches!(w[i], "sudo" | "doas" | "command" | "env" | "nohup" | "time")
                || (w[i].contains('=') && !w[i].starts_with('-')))
        {
            i += 1;
        }
        let Some(&bin) = w.get(i) else { continue };
        let bin = bin.rsplit('/').next().unwrap_or(bin);
        let rest = &w[i + 1..];
        let has = |x: &str| rest.iter().any(|r| *r == x);
        let r = match bin {
            "rm" | "rmdir" | "shred" | "unlink" => Some("This operation may permanently delete files."),
            "dd" | "fdisk" | "parted" | "wipefs" | "diskutil" => Some("Disk-level operation: data may be destroyed."),
            b if b.starts_with("mkfs") => Some("Disk-level operation: data may be destroyed."),
            "shutdown" | "reboot" | "halt" | "poweroff" => Some("This will shut down or restart the machine."),
            "kill" | "killall" | "pkill" => Some("This will terminate running processes."),
            "chmod" | "chown" if has("-R") || has("-r") => Some("Recursive permission/ownership change."),
            "find" if has("-delete") => Some("find -delete permanently removes matches."),
            "git" => {
                if has("reset") && has("--hard") {
                    Some("Discards uncommitted changes permanently.")
                } else if has("clean") {
                    Some("Permanently removes untracked files.")
                } else if has("push") && (has("--force") || has("-f")) {
                    Some("Force-push can overwrite remote history.")
                } else if has("checkout") && has("--") && has(".") {
                    Some("Discards all working-tree changes.")
                } else if has("branch") && has("-D") {
                    Some("Force-deletes a branch, even if unmerged.")
                } else if has("stash") && (has("drop") || has("clear")) {
                    Some("Permanently drops stashed work.")
                } else {
                    None
                }
            }
            _ => None,
        };
        if r.is_some() {
            return r;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags_destructive() {
        assert!(classify("rm -rf ./build").is_some());
        assert!(classify("sudo rm x").is_some());
        assert!(classify("cd /tmp && git reset --hard").is_some());
        assert!(classify("git push --force").is_some());
    }
    #[test]
    fn allows_safe() {
        assert!(classify("git status").is_none());
        assert!(classify("npm run dev").is_none());
        assert!(classify("cd 'a b' && ls -la").is_none());
        assert!(classify("git push origin main").is_none());
    }
}
