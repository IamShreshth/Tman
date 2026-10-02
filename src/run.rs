use std::fs;
use std::path::Path;

pub struct RunItem {
    pub cmd: String,
    pub source: &'static str,
}

pub fn kinds(cwd: &Path) -> Vec<&'static str> {
    let mut k = vec![];
    for (f, n) in [
        ("Cargo.toml", "Rust"),
        ("package.json", "Node"),
        ("Makefile", "Make"),
        ("justfile", "Just"),
        ("Justfile", "Just"),
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
        if cwd.join(f).exists() && !k.contains(&n) {
            k.push(n);
        }
    }
    k
}

/// Discovers project-specific commands instead of hardcoding a list of the world.
pub fn discover(cwd: &Path) -> Vec<RunItem> {
    let mut v = vec![];

    // Node.js (package.json)
    if let Ok(s) = fs::read_to_string(cwd.join("package.json")) {
        if let Ok(j) = serde_json::from_str::<serde_json::Value>(&s) {
            let pm = if cwd.join("pnpm-lock.yaml").exists() { "pnpm" }
                else if cwd.join("yarn.lock").exists() { "yarn" }
                else if cwd.join("bun.lockb").exists() { "bun" } else { "npm" };
            if let Some(scripts) = j.get("scripts").and_then(|s| s.as_object()) {
                for k in scripts.keys() {
                    let cmd = if pm == "npm" && (k == "test" || k == "start") { format!("npm {k}") } else { format!("{pm} run {k}") };
                    v.push(RunItem { cmd, source: "package.json" });
                }
            }
        }
    }

    // Rust (Cargo.toml)
    if cwd.join("Cargo.toml").exists() {
        for c in ["cargo build", "cargo run", "cargo test", "cargo check", "cargo clippy"] {
            v.push(RunItem { cmd: c.into(), source: "Cargo.toml" });
        }
    }

    // Python (.venv, pyproject.toml, requirements.txt, main.py)
    let is_python = cwd.join("pyproject.toml").exists()
        || cwd.join("requirements.txt").exists()
        || cwd.join(".venv").exists()
        || cwd.join("venv").exists()
        || cwd.join("main.py").exists()
        || cwd.join("app.py").exists();
    if is_python {
        if cwd.join(".venv").exists() {
            v.push(RunItem { cmd: "source .venv/bin/activate".into(), source: "Python (venv)" });
        } else if cwd.join("venv").exists() {
            v.push(RunItem { cmd: "source venv/bin/activate".into(), source: "Python (venv)" });
        }
        if cwd.join("poetry.lock").exists() {
            v.push(RunItem { cmd: "poetry run python main.py".into(), source: "Poetry" });
            v.push(RunItem { cmd: "poetry run pytest".into(), source: "Poetry" });
            v.push(RunItem { cmd: "poetry install".into(), source: "Poetry" });
        }
        if cwd.join("main.py").exists() {
            v.push(RunItem { cmd: "python3 main.py".into(), source: "Python" });
        } else if cwd.join("app.py").exists() {
            v.push(RunItem { cmd: "python3 app.py".into(), source: "Python" });
        }
        if cwd.join("pytest.ini").exists() || cwd.join("tests").is_dir() {
            v.push(RunItem { cmd: "pytest".into(), source: "Python (pytest)" });
        }
        if cwd.join("requirements.txt").exists() {
            v.push(RunItem { cmd: "pip install -r requirements.txt".into(), source: "Python" });
        }
    }

    // Go (go.mod)
    if cwd.join("go.mod").exists() {
        for c in ["go run .", "go test ./...", "go build", "go mod tidy"] {
            v.push(RunItem { cmd: c.into(), source: "Go" });
        }
    }

    // Docker (docker-compose, Dockerfile)
    if cwd.join("docker-compose.yml").exists() || cwd.join("compose.yaml").exists() {
        for c in [
            "docker compose up -d",
            "docker compose down",
            "docker compose logs -f",
            "docker compose restart",
            "docker compose ps",
        ] {
            v.push(RunItem { cmd: c.into(), source: "Docker Compose" });
        }
    } else if cwd.join("Dockerfile").exists() {
        v.push(RunItem { cmd: "docker build -t app .".into(), source: "Dockerfile" });
    }

    // Makefile
    if let Ok(s) = fs::read_to_string(cwd.join("Makefile")) {
        for l in s.lines() {
            if let Some((t, _)) = l.split_once(':') {
                if !t.is_empty() && !t.starts_with('.') && !l.starts_with('\t') && !l.contains('=')
                    && t.chars().all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
                {
                    v.push(RunItem { cmd: format!("make {t}"), source: "Makefile" });
                }
            }
        }
    }

    // Justfile
    for name in ["justfile", "Justfile"] {
        if let Ok(s) = fs::read_to_string(cwd.join(name)) {
            for l in s.lines() {
                let trimmed = l.trim();
                if let Some((t, _)) = trimmed.split_once(':') {
                    let target = t.trim();
                    if !target.is_empty() && !target.starts_with('#') && !target.contains('=')
                        && target.chars().all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
                    {
                        v.push(RunItem { cmd: format!("just {target}"), source: "Justfile" });
                    }
                }
            }
            break;
        }
    }

    v
}
