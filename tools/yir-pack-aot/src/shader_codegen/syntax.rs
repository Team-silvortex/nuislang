pub(super) fn replace_identifier_token(haystack: &str, name: &str, replacement: &str) -> String {
    let chars = haystack.chars().collect::<Vec<_>>();
    let name_chars = name.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(haystack.len() + replacement.len());
    let mut i = 0;
    while i < chars.len() {
        let matches_name =
            i + name_chars.len() <= chars.len() && chars[i..i + name_chars.len()] == name_chars[..];
        if matches_name {
            let prev_ok = i == 0 || (!is_identifier_char(chars[i - 1]) && chars[i - 1] != '.');
            let next_ok = i + name_chars.len() == chars.len()
                || !is_identifier_char(chars[i + name_chars.len()]);
            if prev_ok && next_ok {
                out.push_str(replacement);
                i += name_chars.len();
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub(super) fn is_identifier_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

pub(super) fn split_top_level_args(raw: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (idx, ch) in raw.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                args.push(raw[start..idx].trim().to_owned());
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    let tail = raw[start..].trim();
    if !tail.is_empty() {
        args.push(tail.to_owned());
    }
    args
}

pub(super) fn find_matching_paren(expr: &str, open_idx: usize) -> Option<usize> {
    let chars = expr.char_indices().collect::<Vec<_>>();
    let start_pos = chars.iter().position(|(idx, _)| *idx == open_idx)?;
    let mut depth = 0usize;
    for (idx, ch) in chars.into_iter().skip(start_pos) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

#[derive(Clone)]
pub(super) struct FunctionCallMatch {
    pub(super) text: String,
}

pub(super) fn collect_function_calls(expr: &str, prefix: &str) -> Vec<FunctionCallMatch> {
    let mut calls = Vec::new();
    let mut search_start = 0usize;
    while let Some(rel_start) = expr[search_start..].find(prefix) {
        let start = search_start + rel_start;
        let Some(prefix_open_rel) = prefix.find('(') else {
            break;
        };
        let open = start + prefix_open_rel;
        let mut depth = 1usize;
        let mut end = None;
        for (idx, ch) in expr[open + 1..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = Some(open + 1 + idx + ch.len_utf8());
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = end else {
            break;
        };
        calls.push(FunctionCallMatch {
            text: expr[start..end].to_owned(),
        });
        search_start = end;
    }
    calls
}
