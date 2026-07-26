use std::{fs, path::Path};

pub(super) const FILE_NAME: &str = "nuis.nsdb.replay-cursor.lineage-repairs.toml";
const PROTOCOL: &str = "nsdb-yir-replay-cursor-lineage-repair-journal-v6";
const ENTRY_LIMIT: usize = 8;

pub(super) struct RepairJournalSummary {
    pub(super) entry_count: usize,
    pub(super) rotation_generation: u64,
    pub(super) evicted_prefix_hash: String,
    pub(super) window_hash: String,
    pub(super) archived_path: String,
    pub(super) archived_hash: String,
    pub(super) rebuilt_hash: String,
    pub(super) lineage_mutated: bool,
    pub(super) repair_journal_mutated: bool,
    pub(super) event_status: String,
    pub(super) archived_repair_journal_path: String,
    pub(super) archived_repair_journal_hash: String,
}

pub(super) fn validate_repair_journal(
    source: &str,
    expected_lineage_path: &Path,
    lineage_hash: &str,
    provider_dispatch_identity_hash: &str,
) -> Option<RepairJournalSummary> {
    if field(source, "protocol").as_deref() != Some(PROTOCOL)
        || field(source, "entry_limit")?.parse::<usize>().ok()? != ENTRY_LIMIT
        || field(source, "provider_dispatch_identity_hash").as_deref()
            != Some(provider_dispatch_identity_hash)
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
        .map(parse_entry)
        .collect::<Option<Vec<_>>>()?;
    if entries.is_empty() || entries.len() != declared_count || entries.len() > ENTRY_LIMIT {
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
    } else if entries.len() != ENTRY_LIMIT
        || !is_hash(&evicted_prefix_hash)
        || first.sequence != rotation_generation
        || first.previous_event_hash != evicted_prefix_hash
    {
        return None;
    }
    for (index, entry) in entries.iter().enumerate() {
        if !entry_contract_is_valid(entry, provider_dispatch_identity_hash)
            || event_hash(entry) != entry.current_event_hash
        {
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
    if latest.rebuilt_hash != lineage_hash
        || !archived_evidence_is_valid(&latest.archived_path, &latest.archived_hash)
        || !archived_evidence_is_valid(
            &latest.archived_repair_journal_path,
            &latest.archived_repair_journal_hash,
        )
    {
        return None;
    }
    let actual_window_hash = window_hash(
        expected_lineage_path,
        provider_dispatch_identity_hash,
        rotation_generation,
        &evicted_prefix_hash,
        declared_count,
        &first.current_event_hash,
        &latest.current_event_hash,
        lineage_hash,
    );
    if !is_hash(&claimed_window_hash) || claimed_window_hash != actual_window_hash {
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

struct Entry {
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
    provider_dispatch_identity_hash: String,
}

fn parse_entry(source: &str) -> Option<Entry> {
    Some(Entry {
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
        provider_dispatch_identity_hash: field(source, "provider_dispatch_identity_hash")?,
    })
}

fn entry_contract_is_valid(entry: &Entry, provider_dispatch_identity_hash: &str) -> bool {
    matches!(
        entry.status.as_str(),
        "lineage-rebuilt" | "repair-history-recovered"
    ) && entry.repair_journal_mutated
        && (entry.archived_hash == "none" || is_hash(&entry.archived_hash))
        && (entry.archived_repair_journal_hash == "none"
            || is_hash(&entry.archived_repair_journal_hash))
        && is_hash(&entry.rebuilt_hash)
        && (entry.previous_event_hash == "none" || is_hash(&entry.previous_event_hash))
        && is_hash(&entry.current_event_hash)
        && entry.provider_dispatch_identity_hash == provider_dispatch_identity_hash
        && !(entry.status == "repair-history-recovered" && entry.lineage_mutated)
}

fn archived_evidence_is_valid(path: &str, hash: &str) -> bool {
    if path.is_empty() {
        return hash == "none";
    }
    fs::read(path)
        .ok()
        .is_some_and(|bytes| fnv1a64_hex(&bytes) == hash)
}

fn event_hash(entry: &Entry) -> String {
    canonical_hash(&[
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
        entry.provider_dispatch_identity_hash.clone(),
    ])
}

#[allow(clippy::too_many_arguments)]
fn window_hash(
    lineage_path: &Path,
    provider_dispatch_identity_hash: &str,
    rotation_generation: u64,
    evicted_prefix_hash: &str,
    entry_count: usize,
    first_event_hash: &str,
    latest_event_hash: &str,
    lineage_hash: &str,
) -> String {
    canonical_hash(&[
        PROTOCOL.to_owned(),
        lineage_path.display().to_string(),
        provider_dispatch_identity_hash.to_owned(),
        rotation_generation.to_string(),
        evicted_prefix_hash.to_owned(),
        entry_count.to_string(),
        first_event_hash.to_owned(),
        latest_event_hash.to_owned(),
        lineage_hash.to_owned(),
    ])
}

fn canonical_hash(values: &[String]) -> String {
    let mut canonical = Vec::new();
    for value in values {
        canonical.extend_from_slice(value.len().to_string().as_bytes());
        canonical.push(b':');
        canonical.extend_from_slice(value.as_bytes());
    }
    fnv1a64_hex(&canonical)
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

    const DISPATCH_HASH: &str = "0x0123456789abcdef";

    fn journal_source(lineage_path: &Path, rebuilt_hash: &str, dispatch_hash: &str) -> String {
        let mut entry = Entry {
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
            rebuilt_hash: rebuilt_hash.to_owned(),
            provider_dispatch_identity_hash: dispatch_hash.to_owned(),
        };
        entry.current_event_hash = event_hash(&entry);
        let claimed = window_hash(
            lineage_path,
            dispatch_hash,
            0,
            "none",
            1,
            &entry.current_event_hash,
            &entry.current_event_hash,
            rebuilt_hash,
        );
        format!(
            "protocol = \"{PROTOCOL}\"\nlineage_path = \"{}\"\nprovider_dispatch_identity_hash = \"{dispatch_hash}\"\nentry_limit = 8\nrotation_generation = 0\nevicted_prefix_hash = \"none\"\nwindow_hash = \"{claimed}\"\nentry_count = 1\n\n[[entry]]\nsequence = 0\nprevious_event_hash = \"none\"\ncurrent_event_hash = \"{}\"\nstatus = \"repair-history-recovered\"\nlineage_mutated = false\nrepair_journal_mutated = true\narchived_path = \"\"\narchived_hash = \"none\"\narchived_repair_journal_path = \"\"\narchived_repair_journal_hash = \"none\"\nrebuilt_hash = \"{rebuilt_hash}\"\nprovider_dispatch_identity_hash = \"{dispatch_hash}\"\n",
            lineage_path.display(),
            entry.current_event_hash
        )
    }

    #[test]
    fn independently_validates_dispatch_bound_journal() {
        let path = Path::new("lineage.toml");
        let rebuilt_hash = fnv1a64_hex(b"lineage");
        let source = journal_source(path, &rebuilt_hash, DISPATCH_HASH);
        assert!(validate_repair_journal(&source, path, &rebuilt_hash, DISPATCH_HASH).is_some());
    }

    #[test]
    fn rejects_another_dispatch_identity_even_with_valid_old_hashes() {
        let path = Path::new("lineage.toml");
        let rebuilt_hash = fnv1a64_hex(b"lineage");
        let source = journal_source(path, &rebuilt_hash, DISPATCH_HASH);
        assert!(
            validate_repair_journal(&source, path, &rebuilt_hash, "0xfedcba9876543210").is_none()
        );
    }
}
