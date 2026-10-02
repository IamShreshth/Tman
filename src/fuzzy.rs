/// Subsequence fuzzy score. None = no match. Higher = better.
pub fn score(query: &str, text: &str) -> Option<i64> {
    let q: Vec<char> = query.chars().map(lc).collect();
    if q.is_empty() {
        return Some(0);
    }
    let t: Vec<char> = text.chars().collect();
    let tl: Vec<char> = t.iter().copied().map(lc).collect();
    let (mut qi, mut s, mut prev_hit) = (0usize, 0i64, false);
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
    Some(s - (t.len() as i64 / 8))
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
        assert!(score("xyz", "git status").is_none());
        assert!(score("app", "App.tsx") > score("app", "a_big_pipeline.ts"));
        assert_eq!(score("", "anything"), Some(0));
    }
}
