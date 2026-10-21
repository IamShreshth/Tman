use crate::config::Config;

fn zsh_key(k: &str) -> String {
    let k = k.to_lowercase();
    if k == "ctrl-space" || k == "ctrl-shift-space" { return "^@".into(); }
    if let Some(c) = k.strip_prefix("ctrl-shift-").or_else(|| k.strip_prefix("shift-ctrl-")) {
        return format!("^{}", c.to_uppercase());
    }
    if let Some(c) = k.strip_prefix("ctrl-") { return format!("^{}", c.to_uppercase()); }
    if let Some(c) = k.strip_prefix("alt-") { return format!("\\e{c}"); }
    k
}
fn bash_key(k: &str) -> String {
    let k = k.to_lowercase();
    if k == "ctrl-space" || k == "ctrl-shift-space" { return "\\C-@".into(); }
    if let Some(c) = k.strip_prefix("ctrl-shift-").or_else(|| k.strip_prefix("shift-ctrl-")) {
        return format!("\\C-{}", c.to_uppercase());
    }
    if let Some(c) = k.strip_prefix("ctrl-") { return format!("\\C-{c}"); }
    if let Some(c) = k.strip_prefix("alt-") { return format!("\\e{c}"); }
    k
}
fn fish_key(k: &str) -> String {
    let k = k.to_lowercase();
    if k == "ctrl-space" || k == "ctrl-shift-space" { return "-k nul".into(); }
    if let Some(c) = k.strip_prefix("ctrl-shift-").or_else(|| k.strip_prefix("shift-ctrl-")) {
        return format!("\\c{}", c.to_lowercase());
    }
    if let Some(c) = k.strip_prefix("ctrl-") { return format!("\\c{c}"); }
    if let Some(c) = k.strip_prefix("alt-") { return format!("\\e{c}"); }
    k
}

/// Thin integration: the binary only ever prints "<x|i>:<command>"; the shell decides what to do with it.
pub fn script(shell: &str, cfg: &Config) -> Option<String> {
    let key = &cfg.keybindings.open;
    match shell {
        "zsh" => {
            let primary = zsh_key(key);
            let extra = if key.to_lowercase().contains("shift") && key.to_lowercase().contains("ctrl") {
                if key.to_lowercase().ends_with('t') {
                    "\nbindkey '^@' tman-widget 2>/dev/null || true\nbindkey '\\e[84;6u' tman-widget 2>/dev/null || true\nbindkey '\\e[116;6u' tman-widget 2>/dev/null || true\nbindkey '\\e[20;6~' tman-widget 2>/dev/null || true"
                } else {
                    ""
                }
            } else {
                ""
            };
            Some(format!(r#"# tman zsh integration: eval "$(tman init zsh)"
tman() {{
  if [[ $# -eq 0 || "$1" == "ui" ]]; then
    local hist out mode cmd
    hist=$(mktemp "${{TMPDIR:-/tmp}}/tman.XXXXXX") || return
    fc -ln 1 > "$hist" 2>/dev/null
    out=$(TMAN_HISTORY_FILE="$hist" command tman ui)
    rm -f "$hist"
    if [[ -n $out ]]; then
      mode=${{out%%:*}}
      cmd=${{out#*:}}
      if [[ $mode == x ]]; then
        eval "$cmd"
      else
        print -z "$cmd"
      fi
    fi
  else
    command tman "$@"
  fi
}}
tman-widget() {{
  local hist out mode
  hist=$(mktemp "${{TMPDIR:-/tmp}}/tman.XXXXXX") || return
  fc -ln 1 > "$hist" 2>/dev/null
  out=$(TMAN_HISTORY_FILE="$hist" command tman ui)
  rm -f "$hist"
  if [[ -n $out ]]; then
    mode=${{out%%:*}}
    BUFFER=${{out#*:}}
    CURSOR=${{#BUFFER}}
    [[ $mode == x ]] && zle accept-line
  fi
  zle reset-prompt
}}
zle -N tman-widget
bindkey '{}' tman-widget{}
"#, primary, extra))
        },
        "bash" => {
            let primary = bash_key(key);
            let extra = if key.to_lowercase().contains("shift") && key.to_lowercase().contains("ctrl") {
                if key.to_lowercase().ends_with('t') {
                    "\nbind '\"\\C-@\": \"\\C-x\\C-t\\C-x\\C-e\"' 2>/dev/null || true\nbind '\"\\e[84;6u\": \"\\C-x\\C-t\\C-x\\C-e\"' 2>/dev/null || true\nbind '\"\\e[116;6u\": \"\\C-x\\C-t\\C-x\\C-e\"' 2>/dev/null || true\nbind '\"\\e[20;6~\": \"\\C-x\\C-t\\C-x\\C-e\"' 2>/dev/null || true"
                } else {
                    ""
                }
            } else {
                ""
            };
            Some(format!(r#"# tman bash integration: eval "$(tman init bash)"
tman() {{
  if [[ $# -eq 0 || "$1" == "ui" ]]; then
    local hist out mode cmd
    hist=$(mktemp "${{TMPDIR:-/tmp}}/tman.XXXXXX") || return
    history -a; HISTTIMEFORMAT= history | sed 's/^ *[0-9]* *//' > "$hist"
    out=$(TMAN_HISTORY_FILE="$hist" command tman ui)
    rm -f "$hist"
    if [[ -n $out ]]; then
      cmd=${{out#*:}}
      eval "$cmd"
    fi
  else
    command tman "$@"
  fi
}}
__tman_widget() {{
  local hist out mode
  hist=$(mktemp "${{TMPDIR:-/tmp}}/tman.XXXXXX") || return
  history -a; HISTTIMEFORMAT= history | sed 's/^ *[0-9]* *//' > "$hist"
  out=$(TMAN_HISTORY_FILE="$hist" command tman ui)
  rm -f "$hist"
  if [[ -n $out ]]; then
    mode=${{out%%:*}}
    READLINE_LINE=${{out#*:}}
    READLINE_POINT=${{#READLINE_LINE}}
    if [[ $mode == x ]]; then
      bind '"\C-x\C-e": accept-line'
    else
      bind '"\C-x\C-e": ""'
    fi
  else
    bind '"\C-x\C-e": ""'
  fi
}}
bind -x '"\C-x\C-t": __tman_widget'
bind '"\C-x\C-e": ""'
bind '"{primary}": "\C-x\C-t\C-x\C-e"'{extra}
"#))
        },
        "fish" => {
            let primary = fish_key(key);
            Some(format!(r#"# tman fish integration: tman init fish | source
function tman
  if test (count $argv) -eq 0; or test "$argv[1]" = "ui"
    set -l hist (mktemp (set -q TMPDIR; and echo $TMPDIR; or echo /tmp)/tman.XXXXXX); or return
    builtin history > "$hist" 2>/dev/null
    set -l out (TMAN_HISTORY_FILE="$hist" command tman ui)
    rm -f "$hist"
    if test -n "$out"
      set -l mode (string split -m 1 : -- "$out")[1]
      set -l cmd (string split -m 1 : -- "$out")[2]
      if test "$mode" = "x"
        eval "$cmd"
      else
        commandline -r -- "$cmd"
      end
    end
  else
    command tman $argv
  end
end
function __tman_widget
  set -l hist (mktemp (set -q TMPDIR; and echo $TMPDIR; or echo /tmp)/tman.XXXXXX); or return
  builtin history > "$hist" 2>/dev/null
  set -l out (TMAN_HISTORY_FILE="$hist" command tman ui)
  rm -f "$hist"
  if test -n "$out"
    set -l mode (string split -m 1 : -- "$out")[1]
    set -l cmd (string split -m 1 : -- "$out")[2]
    commandline -r -- "$cmd"
    if test "$mode" = "x"
      commandline -f execute
    end
  end
  commandline -f repaint
end
bind {primary} __tman_widget
bind -k nul __tman_widget 2>/dev/null; or true
"#))
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generates_all_shells() {
        let cfg = Config::default();
        assert!(script("zsh", &cfg).unwrap().contains("zle accept-line"));
        assert!(script("bash", &cfg).unwrap().contains("accept-line"));
        assert!(script("fish", &cfg).unwrap().contains("commandline -f execute"));
        assert!(script("csh", &cfg).is_none());
    }
}
