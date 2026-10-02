mod app;
mod config;
mod fs;
mod fuzzy;
mod git;
mod history;
mod init;
mod run;
mod safety;
mod sys;
mod ui;

use std::process::exit;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("init") => match a.get(1).and_then(|s| init::script(s, &config::Config::load())) {
            Some(t) => print!("{t}"),
            None => { eprintln!("usage: tman init <zsh|bash>"); exit(2) }
        },
        Some("config-path") => println!("{}", config::path().display()),
        Some("default-config") => print!("{}", config::DEFAULT_TOML),
        Some("ui") | None => match ui::run() {
            Ok(Some(out)) => println!("{out}"),
            Ok(None) => {}
            Err(e) => { eprintln!("tman: {e}"); exit(1) }
        },
        _ => { eprintln!("usage: tman [ui | init <shell> | config-path | default-config]"); exit(2) }
    }
}
