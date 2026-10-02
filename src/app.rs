use crate::{config::Config, fs as tfs, fuzzy, git, history, run, safety, sys};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// A command that will be handed to the shell. `exec` = run immediately, else just insert for editing.
#[derive(Clone, Debug)]
pub struct Cmd { pub text: String, pub exec: bool, pub preview: bool }
impl Cmd {
    pub fn run(t: impl Into<String>) -> Self { Cmd { text: t.into(), exec: true, preview: false } }
    pub fn edit(t: impl Into<String>) -> Self { Cmd { text: t.into(), exec: false, preview: false } }
    pub fn previewed(mut self) -> Self { self.preview = true; self }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode { Menu, Files, History, Find, Git, GitFiles, Run, Actions(PathBuf), Theme, Bookmarks }
impl Mode {
    pub fn title(&self) -> &'static str {
        match self {
            Mode::Menu => "ACTIONS", Mode::Files => "NAVIGATE", Mode::History => "HISTORY", Mode::Find => "SEARCH",
            Mode::Git => "GIT", Mode::GitFiles => "CHANGED FILES", Mode::Run => "RUN", Mode::Actions(_) => "FILE ACTIONS",
            Mode::Theme => "THEMES", Mode::Bookmarks => "BOOKMARKS",
        }
    }
    pub fn auto_search(&self) -> bool { matches!(self, Mode::History | Mode::Find) }
}

/// What selecting an action does. UI never knows about git/files/etc: it only sees Actions.
#[derive(Clone, Debug)]
pub enum Outcome { Emit(Cmd), Goto(Mode), Theme(String), ToggleBookmark(PathBuf) }

#[derive(Clone, Debug)]
pub struct Action {
    pub name: String,
    pub desc: String,
    pub icon: &'static str,
    pub out: Outcome,
    pub path: Option<PathBuf>,
    pub is_dir: bool,
}
fn act(name: &str, desc: &str, icon: &'static str, out: Outcome) -> Action {
    Action { name: name.into(), desc: desc.into(), icon, out, path: None, is_dir: false }
}

pub enum Overlay { Danger(Cmd, &'static str), Preview(Cmd) }
#[derive(Default)]
pub struct Preview { pub title: String, pub lines: Vec<String> }

pub struct App {
    pub cfg: Config,
    pub start_cwd: PathBuf,
    pub cwd: PathBuf,
    pub git: Option<git::GitInfo>,
    pub mode: Mode,
    stack: Vec<Mode>,
    pub items: Vec<Action>,
    pub filtered: Vec<usize>,
    pub sel: usize,
    pub query: String,
    pub searching: bool,
    pub overlay: Option<Overlay>,
    pub preview: Preview,
    pub entered: Instant,
    pub quit: bool,
    pub result: Option<String>,
    pub state: ListState,
    pub show_hidden: bool,
    hist: Option<Vec<String>>,
}

impl App {
    pub fn new(cfg: Config, cwd: PathBuf) -> App {
        let mut a = App {
            show_hidden: cfg.ui.show_hidden, git: git::info(&cwd), start_cwd: cwd.clone(), cwd, cfg,
            mode: Mode::Menu, stack: vec![], items: vec![], filtered: vec![], sel: 0, query: String::new(),
            searching: false, overlay: None, preview: Preview::default(), entered: Instant::now(),
            quit: false, result: None, state: ListState::default(), hist: None,
        };
        a.rebuild();
        a
    }

    /// 0.0..=1.0 progress of the context-panel slide-in (always 1.0 when animations are off).
    pub fn anim(&self) -> f32 {
        if !self.cfg.ui.animations { return 1.0; }
        (self.entered.elapsed().as_secs_f32() / 0.18).min(1.0)
    }
    pub fn animating(&self) -> bool { self.anim() < 1.0 }

    fn rel(&self, p: &Path) -> String {
        let r = p.strip_prefix(&self.cwd).unwrap_or(p);
        tfs::shell_quote(&if r.as_os_str().is_empty() { ".".into() } else { r.to_string_lossy().into_owned() })
    }

    // ---------- item providers ----------
    fn build(&mut self) -> Vec<Action> {
        let editor = self.cfg.editor();
        match self.mode.clone() {
            Mode::Menu => vec![
                act("Navigate", "Browse and jump between directories", "▸", Outcome::Goto(Mode::Files)),
                act("Bookmarks", "Quick jump to pinned directories", "*", Outcome::Goto(Mode::Bookmarks)),
                act("History", "Search your real shell history", "↺", Outcome::Goto(Mode::History)),
                act("Search", "Find files and folders below here", "⌕", Outcome::Goto(Mode::Find)),
                act("Git", "Repository state and operations", "⎇", Outcome::Goto(Mode::Git)),
                act("Run", "Commands discovered in this project", "▶", Outcome::Goto(Mode::Run)),
                act("Theme", "Choose an accent color theme", "#", Outcome::Goto(Mode::Theme)),
            ],
            Mode::Bookmarks => {
                let mut list = self.cfg.bookmarks.clone();
                if list.is_empty() {
                    let home = std::env::var("HOME").unwrap_or_default();
                    for d in [
                        home.clone(),
                        format!("{home}/Downloads"),
                        format!("{home}/Desktop"),
                        format!("{home}/Documents"),
                    ] {
                        if Path::new(&d).is_dir() && !list.contains(&d) {
                            list.push(d);
                        }
                    }
                }
                list.into_iter().map(|b| {
                    let p = PathBuf::from(&b);
                    let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| b.clone());
                    let cmd = format!("cd -- {}", tfs::shell_quote(&b));
                    Action { path: Some(p), is_dir: true, ..act(&name, &b, "*", Outcome::Emit(Cmd::run(cmd))) }
                }).collect()
            },
            Mode::Theme => {
                let current = self.cfg.ui.accent.trim().to_lowercase();
                let themes = [
                    ("cyan", "Futuristic high-tech cyan (default)", "#"),
                    ("blue", "Electric royal blue", "#"),
                    ("green", "Matrix / terminal emerald green", "#"),
                    ("magenta", "Cyberpunk neon magenta", "#"),
                    ("yellow", "Warm solar amber-yellow", "#"),
                    ("red", "Bold neon crimson red", "#"),
                    ("orange", "Warm retro sunset orange", "#"),
                    ("purple", "Deep vaporwave violet", "#"),
                    ("pink", "Pastel aesthetic rose-pink", "#"),
                    ("peach", "Soft warm aesthetic peach", "#"),
                    ("teal", "Minty fresh sea-teal", "#"),
                    ("white", "Minimalist clean monochrome", "#"),
                ];
                themes.into_iter().map(|(name, desc, icon)| {
                    let is_curr = name == current;
                    let display_name = if is_curr { format!("{name}  [active]") } else { name.to_string() };
                    act(&display_name, desc, icon, Outcome::Theme(name.to_string()))
                }).collect()
            },
            Mode::Files => tfs::list_dir(&self.cwd, self.show_hidden).into_iter().map(|e| self.path_action(e.name, e.path, e.is_dir, &editor)).collect(),
            Mode::Find => tfs::walk(&self.cwd, self.show_hidden, 4, 5000).into_iter().map(|e| self.path_action(e.name, e.path, e.is_dir, &editor)).collect(),
            Mode::History => {
                if self.hist.is_none() { self.hist = Some(history::load()); }
                self.hist.as_ref().unwrap().iter().map(|c| act(c, "", "·", Outcome::Emit(Cmd::run(c.clone())))).collect()
            }
            Mode::Git => {
                if self.git.is_none() { return vec![]; }
                let r = |n: &str, d: &str, c: &str| act(n, d, "⎇", Outcome::Emit(Cmd::run(c)));
                vec![
                    act("Changed files", "Pick a file, then choose what to do", "⎇", Outcome::Goto(Mode::GitFiles)),
                    r("Status", "Short status with branch", "git status -sb"),
                    r("Diff", "Unstaged changes", "git diff"),
                    r("Branches", "All local and remote branches", "git branch -a"),
                    r("Commits", "Last 20 commits", "git log --oneline -n 20"),
                    r("Stash list", "Saved stashes", "git stash list"),
                    act("Pull", "Fast-forward only, with preview", "⎇", Outcome::Emit(Cmd::run("git pull --ff-only").previewed())),
                    act("Push", "Push current branch, with preview", "⎇", Outcome::Emit(Cmd::run("git push").previewed())),
                ]
            }
            Mode::GitFiles => git::changed_files(&self.cwd).into_iter().map(|(st, p)| {
                let name = format!("{:<2} {}", st, p.strip_prefix(&self.cwd).unwrap_or(&p).display());
                Action { path: Some(p.clone()), out: Outcome::Goto(Mode::Actions(p)), ..act(&name, "", "·", Outcome::Goto(Mode::Menu)) }
            }).collect(),
            Mode::Run => run::discover(&self.cwd).into_iter().map(|r| act(&r.cmd, r.source, "▶", Outcome::Emit(Cmd::run(r.cmd.clone())))).collect(),
            Mode::Actions(p) => {
                let q = self.rel(&p);
                let is_dir = p.is_dir();
                let mut v = vec![];
                if is_dir {
                    v.push(act("Go here", "cd into this directory", "▸", Outcome::Emit(Cmd::run(format!("cd -- {}", tfs::shell_quote(&p.to_string_lossy()))))));
                    let is_b = self.cfg.is_bookmarked(&p.to_string_lossy());
                    let label = if is_b { "Unpin bookmark" } else { "Bookmark folder" };
                    let desc = if is_b { "Remove from pinned bookmarks" } else { "Pin to quick bookmarks list" };
                    v.push(act(label, desc, "*", Outcome::ToggleBookmark(p.clone())));
                }
                else { v.push(act("Open", "Open in your editor", "▸", Outcome::Emit(Cmd::run(format!("{editor} {q}"))))); }
                v.push(match clipboard() {
                    Some(c) => act("Copy path", "Copy absolute path to clipboard", "▸", Outcome::Emit(Cmd::run(format!("printf %s {} | {c}", tfs::shell_quote(&p.to_string_lossy()))))),
                    None => act("Print path", "No clipboard tool found", "▸", Outcome::Emit(Cmd::run(format!("echo {}", tfs::shell_quote(&p.to_string_lossy()))))),
                });
                if self.git.is_some() && !is_dir {
                    v.push(act("Git diff", "Changes vs HEAD", "⎇", Outcome::Emit(Cmd::run(format!("git diff HEAD -- {q}")))));
                    v.push(act("Git history", "Commits touching this file", "⎇", Outcome::Emit(Cmd::run(format!("git log --oneline -n 20 -- {q}")))));
                    v.push(act("Blame", "Per-line authorship", "⎇", Outcome::Emit(Cmd::run(format!("git blame {q}")))));
                }
                v.push(act("Rename", "Pre-fills mv; you edit the target", "▸", Outcome::Emit(Cmd::edit(format!("mv -- {q} {q}")))));
                v.push(act("Delete", "Always asks for confirmation", "!", Outcome::Emit(Cmd::run(format!("rm {}-- {q}", if is_dir { "-r " } else { "" })))));
                v
            }
        }
    }

    fn path_action(&self, name: String, path: PathBuf, is_dir: bool, editor: &str) -> Action {
        let q = self.rel(&path);
        let cmd = if is_dir { format!("cd -- {}", tfs::shell_quote(&path.to_string_lossy())) } else { format!("{editor} {q}") };
        let shown = if is_dir && !name.ends_with('/') { format!("{name}/") } else { name };
        let icon = if is_dir {
            if self.cfg.is_bookmarked(&path.to_string_lossy()) { "*" } else { "▸" }
        } else {
            "·"
        };
        Action { path: Some(path), is_dir, ..act(&shown, "", icon, Outcome::Emit(Cmd::run(cmd))) }
    }

    // ---------- state helpers ----------
    pub fn current(&self) -> Option<usize> { self.filtered.get(self.sel).copied() }

    fn rebuild(&mut self) {
        self.items = self.build();
        self.refilter();
    }
    fn refilter(&mut self) {
        let q = &self.query;
        let mut v: Vec<(usize, i64)> = self.items.iter().enumerate().filter_map(|(i, a)| fuzzy::score(q, &a.name).map(|s| (i, s))).collect();
        if !q.is_empty() { v.sort_by(|a, b| b.1.cmp(&a.1)); }
        self.filtered = v.into_iter().map(|x| x.0).collect();
        self.sel = 0;
        self.update_preview();
    }
    fn goto(&mut self, m: Mode) {
        let old = std::mem::replace(&mut self.mode, m);
        self.stack.push(old);
        self.enter_mode();
    }
    fn enter_mode(&mut self) {
        self.query.clear();
        self.searching = self.mode.auto_search();
        self.entered = Instant::now();
        self.rebuild();
    }
    fn back(&mut self) {
        match self.mode {
            Mode::Menu => self.quit = true,
            _ => { self.mode = self.stack.pop().unwrap_or(Mode::Menu); self.enter_mode(); }
        }
    }
    fn chdir(&mut self, p: PathBuf, select: Option<PathBuf>) {
        self.cwd = p;
        self.git = git::info(&self.cwd);
        self.query.clear();
        self.rebuild();
        if let Some(s) = select {
            let pos = self.filtered.iter().position(|&i| self.items[i].path.as_deref() == Some(s.as_path()));
            if let Some(pos) = pos { self.sel = pos; self.update_preview(); }
        }
    }
    fn move_sel(&mut self, d: isize) {
        if self.filtered.is_empty() { return; }
        self.sel = (self.sel as isize + d).clamp(0, self.filtered.len() as isize - 1) as usize;
        self.update_preview();
    }

    // ---------- preview / context ----------
    pub fn update_preview(&mut self) {
        let Some(a) = self.current().map(|i| self.items[i].clone()) else {
            self.preview = if self.mode == Mode::Menu { self.context() } else { Preview::default() };
            return;
        };
        let mode = self.mode.clone();
        self.preview = match mode {
            Mode::Menu => self.context(),
            Mode::GitFiles => {
                let p = a.path.clone().unwrap();
                let mut lines = git::diff_preview(&self.cwd, &p, 80);
                if lines.is_empty() { lines = tfs::head(&p, 80); }
                Preview { title: a.name.trim().to_string(), lines }
            }
            Mode::Files | Mode::Find => {
                let p = a.path.clone().unwrap();
                let lines = if a.is_dir { tfs::list_dir(&p, self.show_hidden).into_iter().take(80).map(|e| if e.is_dir { format!("▸ {}/", e.name) } else { format!("  {}", e.name) }).collect() } else { tfs::head(&p, 80) };
                Preview { title: a.name.clone(), lines }
            }
            Mode::Bookmarks => {
                let p = a.path.clone().unwrap_or_else(|| PathBuf::from(&a.desc));
                let mut lines = vec![
                    format!("Pinned Directory: {}", a.name),
                    format!("Path: {}", p.display()),
                    String::new(),
                    "Press Enter to jump into this folder.".to_string(),
                    "Press [d] or [x] to unpin this bookmark.".to_string(),
                    String::new(),
                    "Contents:".to_string(),
                ];
                if p.is_dir() {
                    for e in tfs::list_dir(&p, false).into_iter().take(25) {
                        lines.push(if e.is_dir { format!("  ▸ {}/", e.name) } else { format!("    {}", e.name) });
                    }
                } else {
                    lines.push("  (Folder not found on disk)".to_string());
                }
                Preview { title: format!("BOOKMARK: {}", a.name), lines }
            }
            Mode::Theme => {
                let color_name = a.name.trim_end_matches("  [active]");
                let lines = vec![
                    format!("Theme: {color_name}"),
                    a.desc.clone(),
                    String::new(),
                    "Press Enter to apply & save this theme.".to_string(),
                    format!("Saved to: {}", crate::config::path().display()),
                    String::new(),
                    "Preview palette:".to_string(),
                    format!("  ❯ [████████]  {color_name}"),
                    "  ⎇ main  +2 ~1 ?0".to_string(),
                ];
                Preview { title: format!("THEME: {}", color_name.to_uppercase()), lines }
            }
            _ => {
                let mut lines = vec![];
                if let Outcome::Emit(c) = &a.out {
                    lines.push("Command".to_string());
                    lines.extend(c.text.lines().map(|l| format!("  {l}")));
                    if let Some(r) = safety::classify(&c.text) { lines.push(String::new()); lines.push(format!("[WARN] {r}")); }
                }
                if !a.desc.is_empty() { lines.push(String::new()); lines.push(a.desc.clone()); }
                Preview { title: a.name.clone(), lines }
            }
        };
    }
    fn context(&self) -> Preview {
        let mut l = vec![];

        l.push("━━━ SYSTEM VITALS ━━━━━━━━━━━━━━━━━━━━".to_string());
        if let Some(batt) = sys::battery() {
            l.push(format!("  Power:   {batt}"));
        }
        l.push(format!("  Uptime:  {}", sys::uptime()));
        l.push(format!("  Load:    {}", sys::cpu_load()));
        if let Some(mem) = sys::memory() {
            l.push(format!("  RAM:     {}", mem));
        }
        if let Some((free, used_pct)) = sys::disk_space(&self.cwd) {
            l.push(format!("  Disk:    {free} free ({used_pct} used)"));
        }
        l.push(String::new());

        l.push("━━━ CONTEXT ━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
        l.push(format!("  Dir:     {}", tfs::display_path(&self.cwd)));
        match &self.git {
            Some(g) => l.push(format!("  Git:     ⎇ {}  +{} ~{} ?{}", g.branch, g.staged, g.modified, g.untracked)),
            None => l.push("  Git:     not a git repository".into()),
        }
        let k = run::kinds(&self.cwd);
        if !k.is_empty() { l.push(format!("  Project: {}", k.join(", "))); }
        if self.cfg.is_bookmarked(&self.cwd.to_string_lossy()) {
            l.push("  Status:  Pinned in Bookmarks".to_string());
        }
        l.push(String::new());

        l.push("━━━ DIRECTORY CONTENTS ━━━━━━━━━━━━━━━".to_string());
        for e in tfs::list_dir(&self.cwd, self.show_hidden).into_iter().take(30) {
            l.push(if e.is_dir { format!("▸ {}/", e.name) } else { format!("  {}", e.name) });
        }
        Preview { title: "MISSION CONTROL".into(), lines: l }
    }

    // ---------- output ----------
    fn emit(&mut self, mut c: Cmd) {
        // The shell's real cwd is start_cwd; if we browsed elsewhere, make that explicit in the command.
        if self.cwd != self.start_cwd && !c.text.starts_with("cd ") {
            c.text = format!("cd -- {} && {}", tfs::shell_quote(&self.cwd.to_string_lossy()), c.text);
        }
        if self.cfg.safety.confirm_destructive {
            if let Some(r) = safety::classify(&c.text) { self.overlay = Some(Overlay::Danger(c, r)); return; }
        }
        if c.preview { self.overlay = Some(Overlay::Preview(c)); } else { self.finish(c); }
    }
    fn finish(&mut self, c: Cmd) {
        self.result = Some(format!("{}:{}", if c.exec { 'x' } else { 'i' }, c.text));
        self.quit = true;
    }
    fn activate(&mut self, edit: bool) {
        let Some(i) = self.current() else { return };
        let a = self.items[i].clone();
        match a.out {
            Outcome::Goto(m) => self.goto(m),
            Outcome::Emit(mut c) => { if edit { c.exec = false; } self.emit(c) }
            Outcome::Theme(c) => {
                self.cfg.set_accent(&c);
                self.rebuild();
            }
            Outcome::ToggleBookmark(p) => {
                self.cfg.toggle_bookmark(p.to_string_lossy().to_string());
                self.rebuild();
            }
        }
    }

    // ---------- input ----------
    pub fn on_key(&mut self, k: KeyEvent) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && matches!(k.code, KeyCode::Char('c') | KeyCode::Char('d')) { self.quit = true; return; }
        if let Some(ov) = self.overlay.take() { self.on_overlay_key(ov, k); return; }
        match k.code {
            KeyCode::Esc => {
                if !self.query.is_empty() { self.query.clear(); self.refilter(); }
                else if self.searching && !self.mode.auto_search() { self.searching = false; }
                else { self.quit = true; }
            }
            KeyCode::Up => self.move_sel(-1),
            KeyCode::Down => self.move_sel(1),
            KeyCode::PageUp => self.move_sel(-10),
            KeyCode::PageDown => self.move_sel(10),
            KeyCode::Home => self.move_sel(-(self.sel as isize)),
            KeyCode::End => self.move_sel(isize::MAX / 2),
            KeyCode::Char('p') if ctrl => self.move_sel(-1),
            KeyCode::Char('n') if ctrl => self.move_sel(1),
            KeyCode::Char('u') if ctrl => { self.query.clear(); self.refilter(); }
            KeyCode::Enter => self.activate(false),
            KeyCode::Tab => {
                let is_path_mode = matches!(self.mode, Mode::Files | Mode::Find);
                match self.current().and_then(|i| self.items[i].path.clone()) {
                    Some(p) if is_path_mode => self.goto(Mode::Actions(p)),
                    _ => self.activate(true),
                }
            }
            KeyCode::Backspace => {
                if self.searching && !self.query.is_empty() { self.query.pop(); self.refilter(); }
                else if self.mode == Mode::Files { self.parent(); } else if !self.searching || self.mode.auto_search() { self.back(); }
            }
            KeyCode::Left if !self.searching || self.query.is_empty() => { if self.mode == Mode::Files { self.parent() } else { self.back() } }
            KeyCode::Right if !self.searching || self.query.is_empty() => match (&self.mode, self.current()) {
                (Mode::Files, Some(i)) => { if self.items[i].is_dir { let p = self.items[i].path.clone().unwrap(); self.chdir(p, None); } }
                (Mode::Menu | Mode::Git | Mode::GitFiles | Mode::Run, Some(_)) => self.activate(false),
                _ => {}
            },
            KeyCode::Char(c) if !ctrl && self.searching => { self.query.push(c); self.refilter(); }
            KeyCode::Char('/') if self.mode != Mode::Menu => self.searching = true,
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('m') => { self.stack.clear(); self.mode = Mode::Menu; self.enter_mode(); }
            KeyCode::Char('b') if !self.searching => {
                if self.mode == Mode::Files {
                    if let Some(i) = self.current() {
                        if let Some(p) = &self.items[i].path {
                            if p.is_dir() {
                                self.cfg.toggle_bookmark(p.to_string_lossy().to_string());
                                self.rebuild();
                                return;
                            }
                        }
                    }
                    self.cfg.toggle_bookmark(self.cwd.to_string_lossy().to_string());
                    self.rebuild();
                } else if self.mode == Mode::Bookmarks {
                    if let Some(i) = self.current() {
                        let path_str = self.items[i].desc.clone();
                        self.cfg.remove_bookmark(&path_str);
                        self.rebuild();
                    }
                }
            }
            KeyCode::Char('d' | 'x') if !self.searching && self.mode == Mode::Bookmarks => {
                if let Some(i) = self.current() {
                    let path_str = self.items[i].desc.clone();
                    self.cfg.remove_bookmark(&path_str);
                    self.rebuild();
                }
            }
            KeyCode::Char('j') => self.move_sel(1),
            KeyCode::Char('k') => self.move_sel(-1),
            KeyCode::Char('.') if self.mode == Mode::Files => { self.show_hidden = !self.show_hidden; self.rebuild(); }
            _ => {}
        }
    }
    fn parent(&mut self) {
        if let Some(p) = self.cwd.parent().map(|p| p.to_path_buf()) {
            let old = self.cwd.clone();
            self.chdir(p, Some(old));
        }
    }
    fn on_overlay_key(&mut self, ov: Overlay, k: KeyEvent) {
        match (ov, k.code) {
            (Overlay::Danger(c, _) | Overlay::Preview(c), KeyCode::Enter) => self.finish(c),
            (Overlay::Danger(mut c, _) | Overlay::Preview(mut c), KeyCode::Char('e' | 'E')) => { c.exec = false; self.finish(c) }
            (_, KeyCode::Esc | KeyCode::Char('n')) => {}
            (ov, _) => self.overlay = Some(ov),
        }
    }
}

fn clipboard() -> Option<&'static str> {
    let path = std::env::var_os("PATH")?;
    [("pbcopy", "pbcopy"), ("wl-copy", "wl-copy"), ("xclip", "xclip -selection clipboard"), ("xsel", "xsel -b -i")]
        .into_iter()
        .find(|(bin, _)| std::env::split_paths(&path).any(|d| d.join(bin).is_file()))
        .map(|(_, cmd)| cmd)
}
