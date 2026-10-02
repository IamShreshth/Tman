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

/// Thin integration: the binary only ever prints "<x|i>:<command>"; the shell decides what to do with it.
pub fn script(shell: &str, cfg: &Config) -> Option<String> {
    let key = &cfg.keybindings.open;
    match shell {
        "zsh" => {
            let primary = zsh_key(key);
            let extra = if key.to_lowercase().contains("shift") && key.to_lowercase().contains("ctrl") {
                if key.to_lowercase().ends_with('t') {
                    "\nbindkey '\\e[84;6u' tman-widget 2>/dev/null || true\nbindkey '\\e[116;6u' tman-widget 2>/dev/null || true\nbindkey '\\e[20;6~' tman-widget 2>/dev/null || true"
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
        "bash" => Some(format!(r#"# tman bash integration: eval "$(tman init bash)"
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
# bash cannot auto-accept a line from a `bind -x` handler, so results are always inserted for you to press Enter.
__tman_widget() {{
  local hist out
  hist=$(mktemp "${{TMPDIR:-/tmp}}/tman.XXXXXX") || return
  history -a; HISTTIMEFORMAT= history | sed 's/^ *[0-9]* *//' > "$hist"
  out=$(TMAN_HISTORY_FILE="$hist" command tman ui)
  rm -f "$hist"
  if [[ -n $out ]]; then
    READLINE_LINE=${{out#*:}}
    READLINE_POINT=${{#READLINE_LINE}}
  fi
}}
bind -x '"{}": __tman_widget'
"#, bash_key(key))),
        _ => None,
    }
}
