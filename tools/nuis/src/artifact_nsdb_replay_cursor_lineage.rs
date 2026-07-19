use std::{fs, path::Path};

const CURSOR_FILE_NAME: &str = "nuis.nsdb.replay-cursor.toml";
const LINEAGE_FILE_NAME: &str = "nuis.nsdb.replay-cursor.lineage.toml";
const LINEAGE_PROTOCOL: &str = "nsdb-yir-replay-cursor-lineage-v1";
const LINEAGE_LIMIT: usize = 8;

pub(crate) struct DebuggerCursorLineageMirror {
    pub(crate) contract: &'static str,
    pub(crate) source_protocol: &'static str,
    pub(crate) path: String,
    pub(crate) ready: bool,
    pub(crate) status: &'static str,
    pub(crate) entry_count: usize,
    pub(crate) latest_hash: Option<String>,
    pub(crate) first_blocker: Option<&'static str>,
    pub(crate) next_action: Option<&'static str>,
    pub(crate) next_command: Option<String>,
}

pub(crate) fn read_debugger_cursor_lineage(output_dir: &Path) -> DebuggerCursorLineageMirror {
    let path = output_dir.join(LINEAGE_FILE_NAME);
    let unavailable = || DebuggerCursorLineageMirror {
        contract: "nuis-debugger-cursor-lineage-mirror-v1",
        source_protocol: LINEAGE_PROTOCOL,
        path: path.display().to_string(),
        ready: false,
        status: "lineage-unavailable",
        entry_count: 0,
        latest_hash: None,
        first_blocker: None,
        next_action: None,
        next_command: None,
    };
    let Ok(source) = fs::read_to_string(&path) else {
        return unavailable();
    };
    let cursor_path = output_dir.join(CURSOR_FILE_NAME);
    let cursor_hash = fs::read(&cursor_path).ok().map(|bytes| fnv1a64_hex(&bytes));
    let (entry_count, latest_hash) =
        match validate_lineage(&source, &cursor_path, cursor_hash.as_deref()) {
            Ok(summary) => summary,
            Err(first_blocker) => {
                return DebuggerCursorLineageMirror {
                    status: "lineage-invalid",
                    first_blocker: Some(first_blocker),
                    next_action: Some("repair-cursor-lineage"),
                    next_command: Some(format!(
                        "nsdb cursor-lineage-repair {} --json",
                        output_dir.display()
                    )),
                    ..unavailable()
                };
            }
        };
    DebuggerCursorLineageMirror {
        contract: "nuis-debugger-cursor-lineage-mirror-v1",
        source_protocol: LINEAGE_PROTOCOL,
        path: path.display().to_string(),
        ready: true,
        status: "lineage-ready",
        entry_count,
        latest_hash: Some(latest_hash),
        first_blocker: None,
        next_action: None,
        next_command: None,
    }
}

fn validate_lineage(
    source: &str,
    expected_cursor_path: &Path,
    cursor_hash: Option<&str>,
) -> Result<(usize, String), &'static str> {
    if field(source, "protocol").as_deref() != Some(LINEAGE_PROTOCOL) {
        return Err("lineage-protocol-invalid");
    }
    if field(source, "entry_limit").and_then(|value| value.parse::<usize>().ok())
        != Some(LINEAGE_LIMIT)
    {
        return Err("lineage-limit-invalid");
    }
    let Some(recorded_cursor_path) = field(source, "cursor_path") else {
        return Err("lineage-cursor-path-missing");
    };
    if !same_path(Path::new(&recorded_cursor_path), expected_cursor_path) {
        return Err("lineage-cursor-path-mismatch");
    }
    let declared_count = field(source, "entry_count")
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or("lineage-entry-count-invalid")?;
    let entries = source
        .split("[[entry]]")
        .skip(1)
        .map(parse_entry)
        .collect::<Option<Vec<_>>>()
        .ok_or("lineage-entry-invalid")?;
    if entries.is_empty() || entries.len() != declared_count || entries.len() > LINEAGE_LIMIT {
        return Err("lineage-entry-count-invalid");
    }
    for (index, entry) in entries.iter().enumerate() {
        if !is_hash(&entry.current_hash)
            || (entry.previous_hash != "none" && !is_hash(&entry.previous_hash))
        {
            return Err("lineage-entry-hash-invalid");
        }
        if let Some(previous) = index.checked_sub(1).and_then(|index| entries.get(index)) {
            if entry.sequence != previous.sequence + 1
                || entry.previous_hash != previous.current_hash
            {
                return Err("lineage-hash-chain-invalid");
            }
        }
    }
    let latest_hash = entries
        .last()
        .ok_or("lineage-entry-count-invalid")?
        .current_hash
        .clone();
    let cursor_hash = cursor_hash.ok_or("lineage-authoritative-cursor-missing")?;
    if cursor_hash != latest_hash {
        return Err("lineage-latest-hash-mismatch");
    }
    Ok((declared_count, latest_hash))
}

struct LineageEntry {
    sequence: u64,
    previous_hash: String,
    current_hash: String,
}

fn parse_entry(source: &str) -> Option<LineageEntry> {
    Some(LineageEntry {
        sequence: field(source, "sequence")?.parse::<u64>().ok()?,
        previous_hash: field(source, "previous_hash")?,
        current_hash: field(source, "current_hash")?,
    })
}

fn field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    let value = source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))?;
    if let Some(value) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return unescape(value);
    }
    Some(value.to_owned())
}

fn unescape(value: &str) -> Option<String> {
    let mut output = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            match character {
                '\\' | '"' => output.push(character),
                _ => return None,
            }
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    (!escaped).then_some(output)
}

fn same_path(recorded: &Path, expected: &Path) -> bool {
    match (recorded.canonicalize(), expected.canonicalize()) {
        (Ok(recorded), Ok(expected)) => recorded == expected,
        _ => recorded == expected,
    }
}

fn is_hash(value: &str) -> bool {
    value.len() == 18
        && value.starts_with("0x")
        && value[2..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nuis-cursor-lineage-{label}-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn mirrors_hash_checked_lineage_without_nsdb_types() {
        let root = temp_dir("ready");
        let cursor_path = root.join(CURSOR_FILE_NAME);
        fs::write(&cursor_path, "cursor-v2").unwrap();
        let first_hash = fnv1a64_hex(b"cursor-v1");
        let latest_hash = fnv1a64_hex(b"cursor-v2");
        fs::write(
            root.join(LINEAGE_FILE_NAME),
            format!(
                "protocol = \"{LINEAGE_PROTOCOL}\"\n\
                 cursor_path = \"{}\"\n\
                 entry_limit = 8\n\
                 entry_count = 2\n\n\
                 [[entry]]\nsequence = 0\nprevious_hash = \"none\"\ncurrent_hash = \"{first_hash}\"\n\
                 after_frame_id = \"frame-0\"\nnext_frame_id = \"frame-1\"\n\n\
                 [[entry]]\nsequence = 1\nprevious_hash = \"{first_hash}\"\ncurrent_hash = \"{latest_hash}\"\n\
                 after_frame_id = \"frame-1\"\nnext_frame_id = \"frame-2\"\n",
                cursor_path.display()
            ),
        )
        .unwrap();

        let mirror = read_debugger_cursor_lineage(&root);
        assert!(mirror.ready);
        assert_eq!(mirror.status, "lineage-ready");
        assert_eq!(mirror.entry_count, 2);
        assert_eq!(mirror.latest_hash.as_deref(), Some(latest_hash.as_str()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_lineage_that_does_not_match_the_authoritative_cursor() {
        let root = temp_dir("stale");
        let cursor_path = root.join(CURSOR_FILE_NAME);
        fs::write(&cursor_path, "cursor-v2").unwrap();
        fs::write(
            root.join(LINEAGE_FILE_NAME),
            format!(
                "protocol = \"{LINEAGE_PROTOCOL}\"\ncursor_path = \"{}\"\nentry_limit = 8\nentry_count = 1\n\n\
                 [[entry]]\nsequence = 0\nprevious_hash = \"none\"\ncurrent_hash = \"{}\"\n\
                 after_frame_id = \"frame-0\"\nnext_frame_id = \"frame-1\"\n",
                cursor_path.display(),
                fnv1a64_hex(b"cursor-v1")
            ),
        )
        .unwrap();

        let mirror = read_debugger_cursor_lineage(&root);
        assert!(!mirror.ready);
        assert_eq!(mirror.status, "lineage-invalid");
        assert_eq!(mirror.first_blocker, Some("lineage-latest-hash-mismatch"));
        assert_eq!(mirror.next_action, Some("repair-cursor-lineage"));
        assert!(mirror
            .next_command
            .as_deref()
            .is_some_and(|command| command.starts_with("nsdb cursor-lineage-repair ")));
        fs::remove_dir_all(root).unwrap();
    }
}
