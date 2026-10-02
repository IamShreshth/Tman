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
        Some("-h") | Some("--help") | Some("help") => {
            println!("tman {} - Context-aware interactive layer over your terminal", env!("CARGO_PKG_VERSION"));
            println!("\nUsage:\n  tman [command]\n\nCommands:\n  ui               Launch interactive TUI HUD (default)\n  init <zsh|bash>  Output shell integration script\n  config-path      Print configuration file path\n  default-config   Output default TOML configuration\n\nOptions:\n  -h, --help       Print help information\n  -V, --version    Print version information");
        }
        Some("-V") | Some("--version") | Some("version") => {
            println!("tman {}", env!("CARGO_PKG_VERSION"));
        }
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
        Some(cmd) => {
            eprintln!("unknown command or flag: {cmd}\nusage: tman [ui | init <shell> | config-path | default-config | --help | --version]");
            exit(2);
        }
    }
}
