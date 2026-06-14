use std::cmp::Reverse;
use std::collections::BTreeSet;

pub(super) fn suggest_similar_name(name: &str, candidates: &BTreeSet<String>) -> Option<String> {
    let prefix = candidate_prefix(name);
    let mut ranked = candidates
        .iter()
        .filter_map(|candidate| {
            let distance = bounded_levenshtein(name, candidate, 3)?;
            let prefix_match = candidate.starts_with(&prefix);
            let shared_prefix = common_prefix_len(name, candidate);
            let len_diff = name.len().abs_diff(candidate.len());
            Some((
                distance,
                !prefix_match,
                Reverse(shared_prefix),
                len_diff,
                candidate.len(),
                candidate.clone(),
            ))
        })
        .collect::<Vec<_>>();
    ranked.sort();
    ranked
        .into_iter()
        .find(|(distance, _, _, _, _, _)| *distance <= 2)
        .map(|(_, _, _, _, _, candidate)| candidate)
}

fn candidate_prefix(name: &str) -> String {
    name.chars().take(2).collect()
}

fn common_prefix_len(lhs: &str, rhs: &str) -> usize {
    lhs.chars()
        .zip(rhs.chars())
        .take_while(|(lhs_ch, rhs_ch)| lhs_ch == rhs_ch)
        .count()
}

fn bounded_levenshtein(lhs: &str, rhs: &str, limit: usize) -> Option<usize> {
    if lhs == rhs {
        return Some(0);
    }
    let lhs_chars = lhs.chars().collect::<Vec<_>>();
    let rhs_chars = rhs.chars().collect::<Vec<_>>();
    if lhs_chars.len().abs_diff(rhs_chars.len()) > limit {
        return None;
    }

    let mut prev = (0..=rhs_chars.len()).collect::<Vec<_>>();
    let mut curr = vec![0; rhs_chars.len() + 1];
    for (i, lhs_ch) in lhs_chars.iter().enumerate() {
        curr[0] = i + 1;
        let mut row_min = curr[0];
        for (j, rhs_ch) in rhs_chars.iter().enumerate() {
            let cost = usize::from(lhs_ch != rhs_ch);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
            row_min = row_min.min(curr[j + 1]);
        }
        if row_min > limit {
            return None;
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    (prev[rhs_chars.len()] <= limit).then_some(prev[rhs_chars.len()])
}
