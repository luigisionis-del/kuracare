//! Sober recognition phrases from the editable file `content/riconoscimenti.it.txt`.

/// The phrases shipped with the app (Luigi edits the file).
pub const PHRASES_IT: &str = include_str!("../../../content/riconoscimenti.it.txt");

/// One phrase per line; `#` starts a comment.
pub fn parse(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// A phrase different from the previous one when there is a choice.
/// `pick(n)` must return an index in `0..n` (random in the app, fixed in tests).
pub fn choose<'a>(
    phrases: &[&'a str],
    previous: Option<&str>,
    pick: impl FnOnce(usize) -> usize,
) -> Option<&'a str> {
    let pool: Vec<&str> = if phrases.len() > 1 {
        phrases
            .iter()
            .copied()
            .filter(|p| Some(*p) != previous)
            .collect()
    } else {
        phrases.to_vec()
    };
    if pool.is_empty() {
        return None;
    }
    pool.get(pick(pool.len()).min(pool.len() - 1)).copied()
}

/// Random phrase for the app.
pub fn random(previous: Option<&str>) -> &'static str {
    use rand::RngExt;
    let phrases = parse(PHRASES_IT);
    choose(&phrases, previous, |n| rand::rng().random_range(0..n)).unwrap_or("Fatto.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_comments_and_blank_lines() {
        assert_eq!(parse("# commento\nUno\n\n  Due  \n"), vec!["Uno", "Due"]);
    }

    #[test]
    fn never_repeats_the_previous_one() {
        let p = ["Uno", "Due"];
        for i in 0..5 {
            assert_eq!(choose(&p, Some("Uno"), |_| i), Some("Due"));
        }
        assert_eq!(choose(&[], None, |_| 0), None);
    }

    #[test]
    fn shipped_file_has_phrases() {
        assert!(parse(PHRASES_IT).len() >= 10);
        assert_ne!(random(None), "Fatto.");
    }
}
