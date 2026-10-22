/// Subsequence fuzzy score across whitespace-separated tokens. None = no match. Higher = better.
pub fn score(query: &str, text: &str) -> Option<i64> {
    let tokens: Vec<&str> = query.split_whitespace().collect();
    if tokens.is_empty() {
        return Some(0);
    }
    let mut total = 0i64;
    for tok in tokens {
        total += score_token(tok, text)?.0;
    }
    Some(total)
}

/// Returns character indices in `text` matched by `query` tokens.
pub fn matched_indices(query: &str, text: &str) -> Vec<usize> {
    let mut idx = Vec::new();
    for tok in query.split_whitespace() {
        if let Some((_, hits)) = score_token(tok, text) {
            idx.extend(hits);
        }
    }
    idx.sort_unstable();
    idx.dedup();
    idx
}

fn score_token(query: &str, text: &str) -> Option<(i64, Vec<usize>)> {
    let q: Vec<char> = query.chars().map(lc).collect();
    if q.is_empty() {
        return Some((0, Vec::new()));
    }
    let t: Vec<char> = text.chars().collect();
    let tl: Vec<char> = t.iter().copied().map(lc).collect();
    let (mut qi, mut s, mut prev_hit) = (0usize, 0i64, false);
    let mut hits = Vec::with_capacity(q.len());
    for i in 0..tl.len() {
        if qi < q.len() && tl[i] == q[qi] {
            s += 1;
            if prev_hit {
                s += 5;
            }
            if i == 0 || !t[i - 1].is_alphanumeric() {
                s += 3;
            }
            prev_hit = true;
            hits.push(i);
            qi += 1;
        } else {
            prev_hit = false;
        }
    }
    if qi < q.len() {
        return None;
    }
    let joined: String = q.iter().collect();
    let hay: String = tl.iter().collect();
    if hay.contains(&joined) {
        s += 20;
    }
    if hay.starts_with(&joined) {
        s += 10;
    }
    Some((s - (t.len() as i64 / 8), hits))
}

fn lc(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subsequence_and_ranking() {
        assert!(score("gst", "git status").is_some());
        assert!(score("status git", "git status").is_some());
        assert!(score("xyz", "git status").is_none());
        assert!(score("app", "App.tsx") > score("app", "a_big_pipeline.ts"));
        assert_eq!(score("", "anything"), Some(0));
        assert_eq!(matched_indices("gt", "git"), vec![0, 2]);
    }
}
