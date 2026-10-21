use crate::fs::shell_quote;
use std::fs;
use std::path::Path;

pub struct RunItem {
    pub cmd: String,
    pub source: String,
}

fn resolve<'a>(cwd: &'a Path, git_root: Option<&'a Path>, file: &str) -> Option<&'a Path> {
    if cwd.join(file).exists() {
        Some(cwd)
    } else if let Some(r) = git_root.filter(|&r| r != cwd && r.join(file).exists()) {
        Some(r)
    } else {
        None
    }
}

fn scoped(cwd: &Path, target_dir: &Path, cmd: impl Into<String>) -> String {
    let c = cmd.into();
    if target_dir == cwd {
        c
    } else {
        format!("cd -- {} && {c}", shell_quote(&target_dir.to_string_lossy()))
    }
}

pub fn kinds(cwd: &Path, git_root: Option<&Path>) -> Vec<&'static str> {
    let mut k = vec![];
    for (f, n) in [
        ("Cargo.toml", "Rust"),
        ("package.json", "Node"),
        ("Makefile", "Make"),
        ("justfile", "Just"),
        ("Justfile", "Just"),
        ("CMakeLists.txt", "CMake"),
        ("pyproject.toml", "Python"),
        ("requirements.txt", "Python"),
        (".venv", "Python (venv)"),
        ("venv", "Python (venv)"),
        ("go.mod", "Go"),
        ("docker-compose.yml", "Docker Compose"),
        ("compose.yaml", "Docker Compose"),
        ("Dockerfile", "Docker"),
        (".git", "Git"),
    ] {
        if resolve(cwd, git_root, f).is_some() && !k.contains(&n) {
            k.push(n);
        }
    }
    k
}

/// Discovers project-specific commands instead of hardcoding a list of the world.
pub fn discover(cwd: &Path, git_root: Option<&Path>) -> Vec<RunItem> {
    let mut v = vec![];

    // Node.js (package.json)
    if let Some(dir) = resolve(cwd, git_root, "package.json") {
        if let Ok(s) = fs::read_to_string(dir.join("package.json")) {
            if let Ok(j) = serde_json::from_str::<serde_json::Value>(&s) {
                let pm = if dir.join("pnpm-lock.yaml").exists() { "pnpm" }
                    else if dir.join("yarn.lock").exists() { "yarn" }
                    else if dir.join("bun.lockb").exists() { "bun" } else { "npm" };
                if let Some(scripts) = j.get("scripts").and_then(|s| s.as_object()) {
                    for (k, val) in scripts {
                        let raw = if pm == "npm" && (k == "test" || k == "start") { format!("npm {k}") } else { format!("{pm} run {k}") };
                        let source = val.as_str().map(|b| format!("package.json: {b}")).unwrap_or_else(|| "package.json".into());
                        v.push(RunItem { cmd: scoped(cwd, dir, raw), source });
                    }
                }
            }
        }
    }

    // Rust (Cargo.toml)
    if resolve(cwd, git_root, "Cargo.toml").is_some() {
        for c in ["cargo build", "cargo run", "cargo test", "cargo check", "cargo clippy"] {
            v.push(RunItem { cmd: c.into(), source: "Cargo.toml".into() });
        }
    }

    // Python (.venv, pyproject.toml, requirements.txt, main.py)
    let py_dir = ["pyproject.toml", "requirements.txt", ".venv", "venv", "main.py", "app.py"]
        .into_iter()
        .find_map(|f| resolve(cwd, git_root, f));
    if let Some(dir) = py_dir {
        if dir.join(".venv").exists() {
            v.push(RunItem { cmd: scoped(cwd, dir, "source .venv/bin/activate"), source: "Python (venv)".into() });
        } else if dir.join("venv").exists() {
            v.push(RunItem { cmd: scoped(cwd, dir, "source venv/bin/activate"), source: "Python (venv)".into() });
        }
        if dir.join("poetry.lock").exists() {
            v.push(RunItem { cmd: scoped(cwd, dir, "poetry run python main.py"), source: "Poetry".into() });
            v.push(RunItem { cmd: scoped(cwd, dir, "poetry run pytest"), source: "Poetry".into() });
            v.push(RunItem { cmd: scoped(cwd, dir, "poetry install"), source: "Poetry".into() });
        }
        if dir.join("main.py").exists() {
            v.push(RunItem { cmd: scoped(cwd, dir, "python3 main.py"), source: "Python".into() });
        } else if dir.join("app.py").exists() {
            v.push(RunItem { cmd: scoped(cwd, dir, "python3 app.py"), source: "Python".into() });
        }
        if dir.join("pytest.ini").exists() || dir.join("tests").is_dir() {
            v.push(RunItem { cmd: scoped(cwd, dir, "pytest"), source: "Python (pytest)".into() });
        }
        if dir.join("requirements.txt").exists() {
            v.push(RunItem { cmd: scoped(cwd, dir, "pip install -r requirements.txt"), source: "Python".into() });
        }
    }

    // Go (go.mod)
    if let Some(dir) = resolve(cwd, git_root, "go.mod") {
        for c in ["go run .", "go test ./...", "go build", "go mod tidy"] {
            v.push(RunItem { cmd: scoped(cwd, dir, c), source: "Go".into() });
        }
    }

    // Docker (docker-compose, Dockerfile)
    if let Some(dir) = resolve(cwd, git_root, "docker-compose.yml").or_else(|| resolve(cwd, git_root, "compose.yaml")) {
        for c in [
            "docker compose up -d",
            "docker compose down",
            "docker compose logs -f",
            "docker compose restart",
            "docker compose ps",
        ] {
            v.push(RunItem { cmd: scoped(cwd, dir, c), source: "Docker Compose".into() });
        }
    } else if let Some(dir) = resolve(cwd, git_root, "Dockerfile") {
        v.push(RunItem { cmd: scoped(cwd, dir, "docker build -t app ."), source: "Dockerfile".into() });
    }

    // Makefile
    if let Some(dir) = resolve(cwd, git_root, "Makefile") {
        if let Ok(s) = fs::read_to_string(dir.join("Makefile")) {
            let lines: Vec<&str> = s.lines().collect();
            for (idx, l) in lines.iter().enumerate() {
                if let Some((t, _)) = l.split_once(':') {
                    if !t.is_empty() && !t.starts_with('.') && !l.starts_with('\t') && !l.contains('=')
                        && t.chars().all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
                    {
                        let source = lines.get(idx + 1).filter(|nl| nl.starts_with('\t')).map(|nl| format!("Makefile: {}", nl.trim())).unwrap_or_else(|| "Makefile".into());
                        v.push(RunItem { cmd: scoped(cwd, dir, format!("make {t}")), source });
                    }
                }
            }
        }
    }

    // CMake (CMakeLists.txt)
    if let Some(dir) = resolve(cwd, git_root, "CMakeLists.txt") {
        for c in ["cmake -B build", "cmake --build build", "ctest --test-dir build"] {
            v.push(RunItem { cmd: scoped(cwd, dir, c), source: "CMakeLists.txt".into() });
        }
    }

    // Justfile
    for name in ["justfile", "Justfile"] {
        if let Some(dir) = resolve(cwd, git_root, name) {
            if let Ok(s) = fs::read_to_string(dir.join(name)) {
                let lines: Vec<&str> = s.lines().collect();
                for (idx, l) in lines.iter().enumerate() {
                    let trimmed = l.trim();
                    if let Some((t, _)) = trimmed.split_once(':') {
                        let target = t.split_whitespace().next().unwrap_or("");
                        if !target.is_empty() && !target.starts_with('#') && !target.contains('=')
                            && target.chars().all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
                        {
                            let source = lines.get(idx + 1).filter(|nl| nl.starts_with(' ') || nl.starts_with('\t')).map(|nl| format!("Justfile: {}", nl.trim())).unwrap_or_else(|| "Justfile".into());
                            v.push(RunItem { cmd: scoped(cwd, dir, format!("just {target}")), source });
                        }
                    }
                }
                break;
            }
        }
    }

    v
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovers_from_git_root_when_in_subdir() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let sub = root.join("src");
        let items = discover(&sub, Some(root));
        assert!(items.iter().any(|i| i.cmd == "cargo test" && i.source == "Cargo.toml"));
    }
}
