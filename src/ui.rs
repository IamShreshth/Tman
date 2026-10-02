use crate::app::{App, Mode, Overlay};
use crate::config::Config;
use crate::fs::display_path;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::fs::OpenOptions;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Restores the terminal on every exit path: normal return, `?` error, panic unwind.
struct TermGuard;
impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        if let Ok(mut tty) = OpenOptions::new().write(true).open("/dev/tty") {
            let _ = execute!(tty, LeaveAlternateScreen, cursor::Show);
        }
    }
}

/// Draws on /dev/tty so stdout stays free to carry the result back to the shell widget.
pub fn run() -> io::Result<Option<String>> {
    let cfg = Config::load();
    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    let stop = Arc::new(AtomicBool::new(false));
    #[cfg(unix)]
    for sig in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT, signal_hook::consts::SIGHUP] {
        let _ = signal_hook::flag::register(sig, Arc::clone(&stop));
    }
    enable_raw_mode()?;
    let _guard = TermGuard;
    let mut out = tty;
    execute!(out, EnterAlternateScreen, cursor::Hide)?;
    let mut term = Terminal::new(CrosstermBackend::new(out))?;
    let mut app = App::new(cfg, std::env::current_dir()?);
    loop {
        term.draw(|f| draw(f, &mut app))?;
        if app.quit || stop.load(Ordering::Relaxed) { break; }
        let wait = if app.animating() { Duration::from_millis(16) } else { Duration::from_millis(250) };
        if event::poll(wait)? {
            if let Event::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Press { app.on_key(k); }
            }
        }
    }
    Ok(app.result.take())
}

fn parse_hex(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}

fn accent(app: &App) -> Color {
    let s = app.cfg.ui.accent.trim().to_lowercase();
    if let Some(c) = parse_hex(&s) {
        return c;
    }
    match s.as_str() {
        "blue" => Color::Rgb(90, 160, 255),
        "green" => Color::Rgb(80, 220, 150),
        "magenta" => Color::Rgb(200, 120, 255),
        "yellow" => Color::Rgb(240, 200, 90),
        "red" | "crimson" => Color::Rgb(255, 95, 105),
        "orange" | "amber" => Color::Rgb(255, 150, 60),
        "purple" | "violet" => Color::Rgb(175, 110, 255),
        "pink" | "rose" => Color::Rgb(255, 120, 190),
        "peach" => Color::Rgb(255, 180, 140),
        "teal" => Color::Rgb(60, 220, 200),
        "white" | "monochrome" => Color::Rgb(235, 240, 245),
        _ => Color::Rgb(70, 215, 230), // cyan
    }
}

fn sel_bg(acc: Color) -> Color {
    match acc {
        Color::Rgb(r, g, b) => Color::Rgb(r / 6 + 10, g / 6 + 10, b / 6 + 10),
        _ => Color::Rgb(20, 44, 56),
    }
}

const DIM: Color = Color::Rgb(110, 120, 135);

fn block<'a>(title: String, c: Color) -> Block<'a> {
    Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)
        .border_style(Style::default().fg(c)).title(Span::styled(format!(" {title} "), Style::default().fg(c).add_modifier(Modifier::BOLD)))
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let acc = accent(app);
    if area.width < 40 || area.height < 10 {
        f.render_widget(Paragraph::new("terminal too small").style(Style::default().fg(DIM)), area);
        return;
    }
    let rows = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3), Constraint::Length(1)]).split(area);

    // header: where am I
    let mut h = vec![Span::styled(display_path(&app.cwd), Style::default().add_modifier(Modifier::BOLD))];
    if let Some(g) = &app.git {
        h.push(Span::styled(format!("   ⎇ {}", g.branch), Style::default().fg(Color::Rgb(80, 220, 150))));
        h.push(Span::styled(format!("  +{} ~{} ?{}", g.staged, g.modified, g.untracked), Style::default().fg(DIM)));
    }
    f.render_widget(Paragraph::new(Line::from(h)).block(block("tman".into(), acc)), rows[0]);

    // body: list + sliding context panel
    let eased = { let t = app.anim(); 1.0 - (1.0 - t).powi(3) };
    let right_pct = if app.cfg.ui.preview && area.width >= 70 { (eased * 45.0) as u16 } else { 0 };
    let cols = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100 - right_pct), Constraint::Percentage(right_pct)]).split(rows[1]);
    draw_list(f, app, cols[0], acc);
    if right_pct > 8 {
        let title = if app.mode == Mode::Menu { "MISSION CONTROL".to_string() } else { format!("PREVIEW  {}", app.preview.title.chars().take(40).collect::<String>()) };
        let lines: Vec<Line> = app.preview.lines.iter().map(|l| style_preview_line(l, acc)).collect();
        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(block(title, DIM)), cols[1]);
    }

    f.render_widget(Paragraph::new(footer(app, acc)), rows[2]);
    if let Some(ov) = &app.overlay { draw_overlay(f, ov, acc, area); }
}

fn style_preview_line(l: &str, acc: Color) -> Line<'static> {
    let s = if l.starts_with("━━━") { Style::default().fg(acc).add_modifier(Modifier::BOLD) }
        else if l.starts_with('+') && !l.starts_with("+++") { Style::default().fg(Color::Rgb(80, 220, 150)) }
        else if l.starts_with('-') && !l.starts_with("---") { Style::default().fg(Color::Rgb(240, 100, 100)) }
        else if l.starts_with("[WARN]") || l.starts_with("WARNING:") { Style::default().fg(Color::Rgb(240, 200, 90)).add_modifier(Modifier::BOLD) }
        else if l.starts_with('▸') { Style::default().fg(acc) }
        else if l.starts_with("  Power:") || l.starts_with("  RAM:") || l.starts_with("  Uptime:") || l.starts_with("  Disk:") || l.starts_with("  Load:") {
            Style::default().fg(Color::Rgb(220, 225, 235))
        }
        else { Style::default() };
    Line::styled(l.to_string(), s)
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect, acc: Color) {
    let title = match &app.mode { Mode::Menu => "ACTIONS".to_string(), m => m.title().to_string() };
    let b = block(title, acc);
    let inner = b.inner(area);
    f.render_widget(b, area);
    let (list_area, input_area) = if app.searching {
        let s = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(1), Constraint::Length(1)]).split(inner);
        (s[0], Some(s[1]))
    } else { (inner, None) };

    let wide = list_area.width > 50;
    let pad = if wide { app.filtered.iter().filter(|&&i| !app.items[i].desc.is_empty()).map(|&i| app.items[i].name.chars().count()).max().unwrap_or(0).min(28) } else { 0 };
    let items: Vec<ListItem> = app.filtered.iter().map(|&i| {
        let a = &app.items[i];
        let name_style = if a.is_dir { Style::default().fg(acc) } else if a.icon == "!" { Style::default().fg(Color::Rgb(240, 100, 100)) } else { Style::default() };
        let mut sp = vec![Span::styled(format!("{} ", a.icon), Style::default().fg(DIM)), Span::styled(a.name.clone(), name_style)];
        if wide && !a.desc.is_empty() {
            let gap = pad.saturating_sub(a.name.chars().count()) + 2;
            sp.push(Span::styled(format!("{}{}", " ".repeat(gap), a.desc), Style::default().fg(DIM)));
        }
        ListItem::new(Line::from(sp))
    }).collect();

    if items.is_empty() {
        let msg = if app.mode == Mode::Git && app.git.is_none() { "Not inside a git repository." }
            else if app.query.is_empty() { "Nothing here." } else { "No matches." };
        f.render_widget(Paragraph::new(msg).style(Style::default().fg(DIM)), list_area);
    } else {
        app.state.select(Some(app.sel));
        let list = List::new(items)
            .highlight_style(Style::default().bg(sel_bg(acc)).fg(acc).add_modifier(Modifier::BOLD))
            .highlight_symbol("❯ ");
        f.render_stateful_widget(list, list_area, &mut app.state);
    }
    if let Some(r) = input_area {
        let n = app.filtered.len();
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled("/ ", Style::default().fg(acc).add_modifier(Modifier::BOLD)),
            Span::raw(app.query.clone()), Span::styled("▏", Style::default().fg(acc)),
            Span::styled(format!("   {n}"), Style::default().fg(DIM)),
        ])), r);
    }
}

fn footer(app: &App, acc: Color) -> Line<'static> {
    let keys: &[(&str, &str)] = if app.overlay.is_some() { &[("Enter", "Confirm"), ("E", "Edit"), ("Esc", "Cancel")] } else {
        match &app.mode {
            Mode::Menu => &[("↑↓", "Move"), ("Enter", "Open"), ("Esc", "Close")],
            Mode::Files => &[("↑↓", "Move"), ("→", "Enter dir"), ("←", "Parent"), ("Enter", "cd / open"), ("b", "Bookmark"), ("Tab", "Actions"), ("/", "Search"), (".", "Hidden"), ("m", "Menu"), ("Esc", "Close")],
            Mode::Bookmarks => &[("↑↓", "Move"), ("Enter", "Jump / cd"), ("d / x", "Delete pin"), ("←", "Back"), ("Esc", "Close")],
            Mode::History => &[("type", "Search"), ("↑↓", "Move"), ("Enter", "Run"), ("Tab", "Edit first"), ("←", "Back"), ("Esc", "Close")],
            Mode::Find => &[("type", "Search"), ("↑↓", "Move"), ("Enter", "cd / open"), ("Tab", "Actions"), ("←", "Back"), ("Esc", "Close")],
            Mode::Theme => &[("↑↓", "Move"), ("Enter", "Apply & Save"), ("←", "Back"), ("Esc", "Close")],
            _ => &[("↑↓", "Move"), ("Enter", "Select"), ("Tab", "Edit first"), ("←", "Back"), ("/", "Search"), ("Esc", "Close")],
        }
    };
    let mut sp = vec![Span::raw(" ")];
    for (k, d) in keys {
        sp.push(Span::styled(k.to_string(), Style::default().fg(acc).add_modifier(Modifier::BOLD)));
        sp.push(Span::styled(format!(" {d}   "), Style::default().fg(DIM)));
    }
    Line::from(sp)
}

fn draw_overlay(f: &mut Frame, ov: &Overlay, acc: Color, area: Rect) {
    let (title, color, cmd, reason) = match ov {
        Overlay::Danger(c, r) => ("WARNING: DESTRUCTIVE ACTION", Color::Rgb(240, 100, 100), c, Some(*r)),
        Overlay::Preview(c) => ("COMMAND PREVIEW", acc, c, None),
    };
    let mut lines: Vec<Line> = vec![Line::raw("")];
    if let Some(r) = reason { lines.push(Line::raw(format!(" {r}"))); lines.push(Line::raw("")); lines.push(Line::styled(" Command:", Style::default().fg(DIM))); }
    for l in cmd.text.lines() { lines.push(Line::styled(format!("   {l}"), Style::default().add_modifier(Modifier::BOLD))); }
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::raw(if cmd.exec { "Execute   " } else { "Insert   " }),
        Span::styled("[E] ", Style::default().fg(color).add_modifier(Modifier::BOLD)), Span::raw("Edit   "),
        Span::styled("[Esc] ", Style::default().fg(color).add_modifier(Modifier::BOLD)), Span::raw("Cancel"),
    ]));
    let w = area.width.saturating_sub(4).min(72);
    let h = (lines.len() as u16 + 2).min(area.height);
    let r = Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height.saturating_sub(h)) / 2, width: w, height: h };
    f.render_widget(Clear, r);
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(block(title.to_string(), color)), r);
}
