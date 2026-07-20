use std::{fs, path::Path};

const CURSOR_FILE_NAME: &str = "nuis.nsdb.replay-cursor.toml";
const LINEAGE_FILE_NAME: &str = "nuis.nsdb.replay-cursor.lineage.toml";
const LINEAGE_PROTOCOL: &str = "nsdb-yir-replay-cursor-lineage-v1";
const REPAIR_JOURNAL_FILE_NAME: &str = "nuis.nsdb.replay-cursor.lineage-repairs.toml";
const REPAIR_JOURNAL_PROTOCOL: &str = "nsdb-yir-replay-cursor-lineage-repair-journal-v5";
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
    pub(crate) repair: DebuggerCursorLineageRepairMirror,
}

pub(crate) struct DebuggerCursorLineageRepairMirror {
    pub(crate) contract: &'static str,
    pub(crate) path: String,
    pub(crate) status: &'static str,
    pub(crate) entry_count: usize,
    pub(crate) rotation_generation: Option<u64>,
    pub(crate) evicted_prefix_hash: Option<String>,
    pub(crate) window_hash: Option<String>,
    pub(crate) latest_mutated: Option<bool>,
    pub(crate) latest_event_status: Option<String>,
    pub(crate) latest_lineage_mutated: Option<bool>,
    pub(crate) latest_repair_journal_mutated: Option<bool>,
    pub(crate) latest_archived_path: Option<String>,
    pub(crate) latest_archived_hash: Option<String>,
    pub(crate) latest_archived_repair_journal_path: Option<String>,
    pub(crate) latest_archived_repair_journal_hash: Option<String>,
    pub(crate) latest_rebuilt_hash: Option<String>,
    pub(crate) action: DebuggerCursorLineageRepairAction,
}

#[derive(Clone)]
pub(crate) struct DebuggerCursorLineageRepairAction {
    pub(crate) first_blocker: Option<&'static str>,
    pub(crate) next_action: Option<&'static str>,
    pub(crate) next_command: Option<String>,
}

pub(crate) fn read_debugger_cursor_lineage(output_dir: &Path) -> DebuggerCursorLineageMirror {
    let path = output_dir.join(LINEAGE_FILE_NAME);
    let repair_unavailable = || DebuggerCursorLineageRepairMirror {
        contract: "nuis-debugger-cursor-lineage-repair-mirror-v1",
        path: output_dir
            .join(REPAIR_JOURNAL_FILE_NAME)
            .display()
            .to_string(),
        status: "repair-history-unavailable",
        entry_count: 0,
        rotation_generation: None,
        evicted_prefix_hash: None,
        window_hash: None,
        latest_mutated: None,
        latest_event_status: None,
        latest_lineage_mutated: None,
        latest_repair_journal_mutated: None,
        latest_archived_path: None,
        latest_archived_hash: None,
        latest_archived_repair_journal_path: None,
        latest_archived_repair_journal_hash: None,
        latest_rebuilt_hash: None,
        action: DebuggerCursorLineageRepairAction::none(),
    };
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
        repair: repair_unavailable(),
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
                        "nuis debug-lineage-repair {} --json",
                        output_dir.display()
                    )),
                    ..unavailable()
                };
            }
        };
    let repair = read_repair_journal(output_dir, &path, &latest_hash);
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
        repair,
    }
}

fn read_repair_journal(
    output_dir: &Path,
    lineage_path: &Path,
    lineage_hash: &str,
) -> DebuggerCursorLineageRepairMirror {
    let path = output_dir.join(REPAIR_JOURNAL_FILE_NAME);
    let unavailable = || DebuggerCursorLineageRepairMirror {
        contract: "nuis-debugger-cursor-lineage-repair-mirror-v1",
        path: path.display().to_string(),
        status: "repair-history-unavailable",
        entry_count: 0,
        rotation_generation: None,
        evicted_prefix_hash: None,
        window_hash: None,
        latest_mutated: None,
        latest_event_status: None,
        latest_lineage_mutated: None,
        latest_repair_journal_mutated: None,
        latest_archived_path: None,
        latest_archived_hash: None,
        latest_archived_repair_journal_path: None,
        latest_archived_repair_journal_hash: None,
        latest_rebuilt_hash: None,
        action: DebuggerCursorLineageRepairAction::none(),
    };
    let Ok(source) = fs::read_to_string(&path) else {
        return unavailable();
    };
    let Some(summary) = validate_repair_journal(&source, lineage_path, lineage_hash) else {
        return DebuggerCursorLineageRepairMirror {
            status: "repair-history-invalid",
            action: DebuggerCursorLineageRepairAction {
                first_blocker: Some("repair-history-contract-invalid"),
                next_action: Some("repair-cursor-lineage-history"),
                next_command: Some(format!(
                    "nuis debug-lineage-repair {} --json",
                    output_dir.display()
                )),
            },
            ..unavailable()
        };
    };
    DebuggerCursorLineageRepairMirror {
        contract: "nuis-debugger-cursor-lineage-repair-mirror-v1",
        path: path.display().to_string(),
        status: "repair-history-ready",
        entry_count: summary.entry_count,
        rotation_generation: Some(summary.rotation_generation),
        evicted_prefix_hash: (summary.evicted_prefix_hash != "none")
            .then_some(summary.evicted_prefix_hash),
        window_hash: Some(summary.window_hash),
        latest_mutated: Some(summary.lineage_mutated || summary.repair_journal_mutated),
        latest_event_status: Some(summary.event_status),
        latest_lineage_mutated: Some(summary.lineage_mutated),
        latest_repair_journal_mutated: Some(summary.repair_journal_mutated),
        latest_archived_path: (!summary.archived_path.is_empty()).then_some(summary.archived_path),
        latest_archived_hash: (summary.archived_hash != "none").then_some(summary.archived_hash),
        latest_archived_repair_journal_path: (!summary.archived_repair_journal_path.is_empty())
            .then_some(summary.archived_repair_journal_path),
        latest_archived_repair_journal_hash: (summary.archived_repair_journal_hash != "none")
            .then_some(summary.archived_repair_journal_hash),
        latest_rebuilt_hash: Some(summary.rebuilt_hash),
        action: DebuggerCursorLineageRepairAction::none(),
    }
}

impl DebuggerCursorLineageRepairAction {
    fn none() -> Self {
        Self {
            first_blocker: None,
            next_action: None,
            next_command: None,
        }
    }
}

struct RepairJournalSummary {
    entry_count: usize,
    rotation_generation: u64,
    evicted_prefix_hash: String,
    window_hash: String,
    archived_path: String,
    archived_hash: String,
    rebuilt_hash: String,
    lineage_mutated: bool,
    repair_journal_mutated: bool,
    event_status: String,
    archived_repair_journal_path: String,
    archived_repair_journal_hash: String,
}

fn validate_repair_journal(
    source: &str,
    expected_lineage_path: &Path,
    lineage_hash: &str,
) -> Option<RepairJournalSummary> {
    if field(source, "protocol").as_deref() != Some(REPAIR_JOURNAL_PROTOCOL)
        || field(source, "entry_limit")?.parse::<usize>().ok()? != LINEAGE_LIMIT
        || !same_path(
            Path::new(&field(source, "lineage_path")?),
            expected_lineage_path,
        )
    {
        return None;
    }
    let declared_count = field(source, "entry_count")?.parse::<usize>().ok()?;
    let rotation_generation = field(source, "rotation_generation")?.parse::<u64>().ok()?;
    let evicted_prefix_hash = field(source, "evicted_prefix_hash")?;
    let claimed_window_hash = field(source, "window_hash")?;
    let entries = source
        .split("[[entry]]")
        .skip(1)
        .map(parse_repair_entry)
        .collect::<Option<Vec<_>>>()?;
    if entries.is_empty() || entries.len() != declared_count || entries.len() > LINEAGE_LIMIT {
        return None;
    }
    let first = entries.first()?;
    if rotation_generation == 0 {
        if evicted_prefix_hash != "none"
            || first.sequence != 0
            || first.previous_event_hash != "none"
        {
            return None;
        }
    } else if entries.len() != LINEAGE_LIMIT
        || !is_hash(&evicted_prefix_hash)
        || first.sequence != rotation_generation
        || first.previous_event_hash != evicted_prefix_hash
    {
        return None;
    }
    for (index, entry) in entries.iter().enumerate() {
        if !matches!(
            entry.status.as_str(),
            "lineage-rebuilt" | "repair-history-recovered"
        ) || !entry.repair_journal_mutated
            || (entry.archived_hash != "none" && !is_hash(&entry.archived_hash))
            || (entry.archived_repair_journal_hash != "none"
                && !is_hash(&entry.archived_repair_journal_hash))
            || !is_hash(&entry.rebuilt_hash)
            || (entry.previous_event_hash != "none" && !is_hash(&entry.previous_event_hash))
            || !is_hash(&entry.current_event_hash)
        {
            return None;
        }
        if entry.status == "repair-history-recovered" && entry.lineage_mutated {
            return None;
        }
        if repair_event_hash(entry) != entry.current_event_hash {
            return None;
        }
        if let Some(previous) = index.checked_sub(1).and_then(|index| entries.get(index)) {
            if entry.sequence != previous.sequence + 1
                || entry.previous_event_hash != previous.current_event_hash
            {
                return None;
            }
        }
    }
    let latest = entries.last()?;
    if latest.rebuilt_hash != lineage_hash {
        return None;
    }
    if !latest.archived_path.is_empty() {
        let bytes = fs::read(&latest.archived_path).ok()?;
        if fnv1a64_hex(&bytes) != latest.archived_hash {
            return None;
        }
    } else if latest.archived_hash != "none" {
        return None;
    }
    if !latest.archived_repair_journal_path.is_empty() {
        let bytes = fs::read(&latest.archived_repair_journal_path).ok()?;
        if fnv1a64_hex(&bytes) != latest.archived_repair_journal_hash {
            return None;
        }
    } else if latest.archived_repair_journal_hash != "none" {
        return None;
    }
    let window_hash = repair_window_hash(
        expected_lineage_path,
        rotation_generation,
        &evicted_prefix_hash,
        declared_count,
        &first.current_event_hash,
        &latest.current_event_hash,
        lineage_hash,
    );
    if !is_hash(&claimed_window_hash) || claimed_window_hash != window_hash {
        return None;
    }
    Some(RepairJournalSummary {
        entry_count: declared_count,
        rotation_generation,
        evicted_prefix_hash,
        window_hash: claimed_window_hash,
        archived_path: latest.archived_path.clone(),
        archived_hash: latest.archived_hash.clone(),
        rebuilt_hash: latest.rebuilt_hash.clone(),
        lineage_mutated: latest.lineage_mutated,
        repair_journal_mutated: latest.repair_journal_mutated,
        event_status: latest.status.clone(),
        archived_repair_journal_path: latest.archived_repair_journal_path.clone(),
        archived_repair_journal_hash: latest.archived_repair_journal_hash.clone(),
    })
}

struct RepairJournalEntry {
    sequence: u64,
    previous_event_hash: String,
    current_event_hash: String,
    status: String,
    lineage_mutated: bool,
    repair_journal_mutated: bool,
    archived_path: String,
    archived_hash: String,
    archived_repair_journal_path: String,
    archived_repair_journal_hash: String,
    rebuilt_hash: String,
}

fn parse_repair_entry(source: &str) -> Option<RepairJournalEntry> {
    Some(RepairJournalEntry {
        sequence: field(source, "sequence")?.parse::<u64>().ok()?,
        previous_event_hash: field(source, "previous_event_hash")?,
        current_event_hash: field(source, "current_event_hash")?,
        status: field(source, "status")?,
        lineage_mutated: field(source, "lineage_mutated")?.parse::<bool>().ok()?,
        repair_journal_mutated: field(source, "repair_journal_mutated")?
            .parse::<bool>()
            .ok()?,
        archived_path: field(source, "archived_path")?,
        archived_hash: field(source, "archived_hash")?,
        archived_repair_journal_path: field(source, "archived_repair_journal_path")?,
        archived_repair_journal_hash: field(source, "archived_repair_journal_hash")?,
        rebuilt_hash: field(source, "rebuilt_hash")?,
    })
}

fn repair_event_hash(entry: &RepairJournalEntry) -> String {
    let mut canonical = Vec::new();
    for value in [
        entry.sequence.to_string(),
        entry.previous_event_hash.clone(),
        entry.status.clone(),
        entry.lineage_mutated.to_string(),
        entry.repair_journal_mutated.to_string(),
        entry.archived_path.clone(),
        entry.archived_hash.clone(),
        entry.archived_repair_journal_path.clone(),
        entry.archived_repair_journal_hash.clone(),
        entry.rebuilt_hash.clone(),
    ] {
        canonical.extend_from_slice(value.len().to_string().as_bytes());
        canonical.push(b':');
        canonical.extend_from_slice(value.as_bytes());
    }
    fnv1a64_hex(&canonical)
}

fn repair_window_hash(
    lineage_path: &Path,
    rotation_generation: u64,
    evicted_prefix_hash: &str,
    entry_count: usize,
    first_event_hash: &str,
    latest_event_hash: &str,
    lineage_hash: &str,
) -> String {
    let mut canonical = Vec::new();
    for value in [
        REPAIR_JOURNAL_PROTOCOL.to_owned(),
        lineage_path.display().to_string(),
        rotation_generation.to_string(),
        evicted_prefix_hash.to_owned(),
        entry_count.to_string(),
        first_event_hash.to_owned(),
        latest_event_hash.to_owned(),
        lineage_hash.to_owned(),
    ] {
        canonical.extend_from_slice(value.len().to_string().as_bytes());
        canonical.push(b':');
        canonical.extend_from_slice(value.as_bytes());
    }
    fnv1a64_hex(&canonical)
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
            .is_some_and(|command| command.starts_with("nuis debug-lineage-repair ")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn repair_mirror_rejects_rewritten_event_content() {
        let root = temp_dir("repair-tamper");
        let lineage_path = root.join(LINEAGE_FILE_NAME);
        let rebuilt_hash = fnv1a64_hex(b"lineage");
        let mut entry = RepairJournalEntry {
            sequence: 0,
            previous_event_hash: "none".to_owned(),
            current_event_hash: String::new(),
            status: "repair-history-recovered".to_owned(),
            lineage_mutated: false,
            repair_journal_mutated: true,
            archived_path: String::new(),
            archived_hash: "none".to_owned(),
            archived_repair_journal_path: String::new(),
            archived_repair_journal_hash: "none".to_owned(),
            rebuilt_hash: rebuilt_hash.clone(),
        };
        entry.current_event_hash = repair_event_hash(&entry);
        let claimed_window_hash = repair_window_hash(
            &lineage_path,
            0,
            "none",
            1,
            &entry.current_event_hash,
            &entry.current_event_hash,
            &rebuilt_hash,
        );
        let source = format!(
            "protocol = \"{REPAIR_JOURNAL_PROTOCOL}\"\nlineage_path = \"{}\"\nentry_limit = 8\nrotation_generation = 0\nevicted_prefix_hash = \"none\"\nwindow_hash = \"{claimed_window_hash}\"\nentry_count = 1\n\n\
             [[entry]]\nsequence = 0\nprevious_event_hash = \"none\"\ncurrent_event_hash = \"{}\"\n\
             status = \"repair-history-recovered\"\nlineage_mutated = false\nrepair_journal_mutated = true\n\
             archived_path = \"\"\narchived_hash = \"none\"\narchived_repair_journal_path = \"\"\n\
             archived_repair_journal_hash = \"none\"\nrebuilt_hash = \"{rebuilt_hash}\"\n",
            lineage_path.display(),
            entry.current_event_hash
        );

        let summary = validate_repair_journal(&source, &lineage_path, &rebuilt_hash).unwrap();
        assert_eq!(summary.rotation_generation, 0);
        assert_eq!(summary.evicted_prefix_hash, "none");
        assert_eq!(summary.window_hash, claimed_window_hash);
        let reformatted = format!("\n{source}\n");
        let reformatted_summary =
            validate_repair_journal(&reformatted, &lineage_path, &rebuilt_hash).unwrap();
        assert_eq!(summary.window_hash, reformatted_summary.window_hash);
        let tampered = source.replace(
            "status = \"repair-history-recovered\"",
            "status = \"lineage-rebuilt\"",
        );
        assert!(validate_repair_journal(&tampered, &lineage_path, &rebuilt_hash).is_none());
        let mismatched = source.replace(
            &format!("window_hash = \"{claimed_window_hash}\""),
            "window_hash = \"0x0000000000000000\"",
        );
        assert!(validate_repair_journal(&mismatched, &lineage_path, &rebuilt_hash).is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn repair_mirror_rejects_truncated_rotated_window() {
        let root = temp_dir("repair-prefix");
        let lineage_path = root.join(LINEAGE_FILE_NAME);
        let rebuilt_hash = fnv1a64_hex(b"lineage");
        let prefix_hash = fnv1a64_hex(b"evicted-prefix");
        let mut previous_hash = prefix_hash.clone();
        let mut first_event_hash = None;
        let mut rendered_entries = String::new();
        for sequence in 4..12 {
            let mut entry = RepairJournalEntry {
                sequence,
                previous_event_hash: previous_hash,
                current_event_hash: String::new(),
                status: "repair-history-recovered".to_owned(),
                lineage_mutated: false,
                repair_journal_mutated: true,
                archived_path: String::new(),
                archived_hash: "none".to_owned(),
                archived_repair_journal_path: String::new(),
                archived_repair_journal_hash: "none".to_owned(),
                rebuilt_hash: rebuilt_hash.clone(),
            };
            entry.current_event_hash = repair_event_hash(&entry);
            if first_event_hash.is_none() {
                first_event_hash = Some(entry.current_event_hash.clone());
            }
            rendered_entries.push_str(&format!(
                "\n[[entry]]\nsequence = {}\nprevious_event_hash = \"{}\"\ncurrent_event_hash = \"{}\"\n\
                 status = \"repair-history-recovered\"\nlineage_mutated = false\nrepair_journal_mutated = true\n\
                 archived_path = \"\"\narchived_hash = \"none\"\narchived_repair_journal_path = \"\"\n\
                 archived_repair_journal_hash = \"none\"\nrebuilt_hash = \"{rebuilt_hash}\"\n",
                entry.sequence, entry.previous_event_hash, entry.current_event_hash
            ));
            previous_hash = entry.current_event_hash;
        }
        let claimed_window_hash = repair_window_hash(
            &lineage_path,
            4,
            &prefix_hash,
            8,
            first_event_hash.as_deref().unwrap(),
            &previous_hash,
            &rebuilt_hash,
        );
        let source = format!(
            "protocol = \"{REPAIR_JOURNAL_PROTOCOL}\"\nlineage_path = \"{}\"\nentry_limit = 8\n\
             rotation_generation = 4\nevicted_prefix_hash = \"{prefix_hash}\"\nwindow_hash = \"{claimed_window_hash}\"\nentry_count = 8\n{rendered_entries}",
            lineage_path.display()
        );
        let summary = validate_repair_journal(&source, &lineage_path, &rebuilt_hash).unwrap();
        assert_eq!(summary.rotation_generation, 4);
        assert_eq!(summary.evicted_prefix_hash, prefix_hash);
        assert_eq!(summary.window_hash, claimed_window_hash);

        let counted = source.replacen("entry_count = 8", "entry_count = 7", 1);
        let first = counted.find("[[entry]]").unwrap();
        let second = first
            + "[[entry]]".len()
            + counted[first + "[[entry]]".len()..]
                .find("[[entry]]")
                .unwrap();
        let truncated = format!("{}{}", &counted[..first], &counted[second..]);
        assert!(validate_repair_journal(&truncated, &lineage_path, &rebuilt_hash).is_none());
        fs::remove_dir_all(root).unwrap();
    }
}
