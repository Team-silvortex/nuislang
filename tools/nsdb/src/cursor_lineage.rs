use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::cursor_lineage_repair_journal as repair_journal;
use crate::{cursor::persist_validated_content_atomically, provider_sample_payload::fnv1a64_hex};

const LINEAGE_PROTOCOL: &str = "nsdb-yir-replay-cursor-lineage-v1";
const LINEAGE_LIMIT: usize = 8;
const CURSOR_FILE_NAME: &str = "nuis.nsdb.replay-cursor.toml";

#[derive(Debug)]
pub(super) struct CursorLineageRepairReport {
    pub(super) contract: &'static str,
    pub(super) status: &'static str,
    pub(super) mutated: bool,
    pub(super) lineage_mutated: bool,
    pub(super) repair_journal_mutated: bool,
    pub(super) cursor_path: String,
    pub(super) lineage_path: String,
    pub(super) archived_path: Option<String>,
    pub(super) repair_journal_path: String,
    pub(super) archived_repair_journal_path: Option<String>,
    pub(super) entry_count: usize,
    pub(super) latest_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CursorLineageEntry {
    sequence: u64,
    previous_hash: String,
    current_hash: String,
    after_frame_id: String,
    next_frame_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CursorLineage {
    cursor_path: String,
    entries: Vec<CursorLineageEntry>,
}

pub(super) fn record_cursor_lineage(
    cursor_path: &Path,
    previous_content: Option<&str>,
    current_content: &str,
    after_frame_id: &str,
    next_frame_id: &str,
) -> Result<(), String> {
    let sidecar_path = cursor_lineage_path(cursor_path)?;
    let cursor_path_text = cursor_path.display().to_string();
    let mut lineage = if sidecar_path.exists() {
        load_cursor_lineage(&sidecar_path, &cursor_path_text)?
    } else {
        CursorLineage {
            cursor_path: cursor_path_text.clone(),
            entries: Vec::new(),
        }
    };
    let previous_hash = previous_content
        .map(|content| fnv1a64_hex(content.as_bytes()))
        .unwrap_or_else(|| "none".to_owned());
    if let Some(latest) = lineage.entries.last() {
        if latest.current_hash != previous_hash {
            return Err(format!(
                "cursor lineage predecessor hash `{}` does not match cursor hash `{previous_hash}`",
                latest.current_hash
            ));
        }
    }
    let sequence = lineage
        .entries
        .last()
        .map(|entry| entry.sequence + 1)
        .unwrap_or(0);
    lineage.entries.push(CursorLineageEntry {
        sequence,
        previous_hash,
        current_hash: fnv1a64_hex(current_content.as_bytes()),
        after_frame_id: after_frame_id.to_owned(),
        next_frame_id: next_frame_id.to_owned(),
    });
    if lineage.entries.len() > LINEAGE_LIMIT {
        lineage
            .entries
            .drain(..lineage.entries.len() - LINEAGE_LIMIT);
    }
    let content = render_cursor_lineage(&lineage);
    persist_validated_content_atomically(&sidecar_path, &content, |temporary| {
        load_cursor_lineage(temporary, &cursor_path_text).map(|_| ())
    })
}

pub(super) fn repair_cursor_lineage(
    output_dir: &Path,
    manifest: &Path,
) -> Result<CursorLineageRepairReport, String> {
    let cursor_path = output_dir.join(CURSOR_FILE_NAME);
    let cursor_content = fs::read_to_string(&cursor_path).map_err(|error| {
        format!(
            "failed to read authoritative replay cursor `{}`: {error}",
            cursor_path.display()
        )
    })?;
    let control = crate::cursor::load_replay_cursor(&cursor_path, manifest)?;
    let after_frame_id = control
        .resume_after_frame_id
        .as_deref()
        .ok_or_else(|| "authoritative replay cursor has no after frame".to_owned())?;
    let next_frame_id = control
        .resume_next_frame_id
        .as_deref()
        .ok_or_else(|| "authoritative replay cursor has no next frame".to_owned())?;
    let current_hash = fnv1a64_hex(cursor_content.as_bytes());
    let lineage_path = cursor_lineage_path(&cursor_path)?;
    let expected_cursor_path = cursor_path.display().to_string();
    let lineage = load_cursor_lineage(&lineage_path, &expected_cursor_path).ok();
    let lineage_ready = lineage.as_ref().is_some_and(|lineage| {
        lineage
            .entries
            .last()
            .is_some_and(|entry| entry.current_hash == current_hash)
    });
    let repair_preflight = repair_journal::preflight(output_dir, &lineage_path)?;
    if lineage_ready {
        if repair_preflight.archived_path.is_some() {
            repair_journal::record(
                &repair_preflight,
                &lineage_path,
                "repair-history-recovered",
                false,
                None,
                &current_hash,
            )?;
            return Ok(CursorLineageRepairReport {
                contract: "nsdb-yir-replay-cursor-lineage-repair-v2",
                status: "repair-history-recovered",
                mutated: true,
                lineage_mutated: false,
                repair_journal_mutated: true,
                cursor_path: expected_cursor_path,
                lineage_path: lineage_path.display().to_string(),
                archived_path: None,
                repair_journal_path: repair_preflight.path.display().to_string(),
                archived_repair_journal_path: repair_preflight
                    .archived_path
                    .map(|path| path.display().to_string()),
                entry_count: lineage
                    .as_ref()
                    .map(|lineage| lineage.entries.len())
                    .unwrap_or(0),
                latest_hash: current_hash,
            });
        }
        return Ok(CursorLineageRepairReport {
            contract: "nsdb-yir-replay-cursor-lineage-repair-v2",
            status: "already-ready",
            mutated: false,
            lineage_mutated: false,
            repair_journal_mutated: false,
            cursor_path: expected_cursor_path,
            lineage_path: lineage_path.display().to_string(),
            archived_path: None,
            repair_journal_path: repair_preflight.path.display().to_string(),
            archived_repair_journal_path: None,
            entry_count: lineage
                .as_ref()
                .map(|lineage| lineage.entries.len())
                .unwrap_or(0),
            latest_hash: current_hash,
        });
    }
    let archived_path = if lineage_path.exists() {
        Some(archive_invalid_lineage(&lineage_path)?)
    } else {
        None
    };
    record_cursor_lineage(
        &cursor_path,
        None,
        &cursor_content,
        after_frame_id,
        next_frame_id,
    )?;
    let rebuilt = load_cursor_lineage(&lineage_path, &expected_cursor_path)?;
    repair_journal::record(
        &repair_preflight,
        &lineage_path,
        "lineage-rebuilt",
        true,
        archived_path.as_deref(),
        &current_hash,
    )?;
    Ok(CursorLineageRepairReport {
        contract: "nsdb-yir-replay-cursor-lineage-repair-v2",
        status: "lineage-rebuilt",
        mutated: true,
        lineage_mutated: true,
        repair_journal_mutated: true,
        cursor_path: expected_cursor_path,
        lineage_path: lineage_path.display().to_string(),
        archived_path: archived_path.map(|path| path.display().to_string()),
        repair_journal_path: repair_preflight.path.display().to_string(),
        archived_repair_journal_path: repair_preflight
            .archived_path
            .map(|path| path.display().to_string()),
        entry_count: rebuilt.entries.len(),
        latest_hash: current_hash,
    })
}

fn archive_invalid_lineage(path: &Path) -> Result<PathBuf, String> {
    archive_invalid_sidecar(path, "cursor lineage")
}

fn archive_invalid_sidecar(path: &Path, description: &str) -> Result<PathBuf, String> {
    let source = fs::read(path).map_err(|error| {
        format!(
            "failed to read invalid {description} `{}`: {error}",
            path.display()
        )
    })?;
    let hash = fnv1a64_hex(&source).trim_start_matches("0x").to_owned();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{description} path `{}` has no file name", path.display()))?;
    let stem = file_name.strip_suffix(".toml").unwrap_or(file_name);
    for suffix in 0..16 {
        let suffix = if suffix == 0 {
            String::new()
        } else {
            format!("-{suffix}")
        };
        let archive = path.with_file_name(format!("{stem}.invalid-{hash}{suffix}.toml"));
        if archive.exists() {
            continue;
        }
        fs::rename(path, &archive).map_err(|error| {
            format!(
                "failed to archive invalid {description} `{}`: {error}",
                path.display(),
            )
        })?;
        return Ok(archive);
    }
    Err(format!(
        "failed to reserve an archive path for invalid {description} `{}`",
        path.display()
    ))
}

fn cursor_lineage_path(cursor_path: &Path) -> Result<PathBuf, String> {
    let file_name = cursor_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("cursor path `{}` has no file name", cursor_path.display()))?;
    let lineage_name = file_name
        .strip_suffix(".toml")
        .map(|stem| format!("{stem}.lineage.toml"))
        .unwrap_or_else(|| format!("{file_name}.lineage.toml"));
    Ok(cursor_path.with_file_name(lineage_name))
}

fn render_cursor_lineage(lineage: &CursorLineage) -> String {
    let mut output = format!(
        "protocol = \"{LINEAGE_PROTOCOL}\"\n\
         cursor_path = \"{}\"\n\
         entry_limit = {LINEAGE_LIMIT}\n\
         entry_count = {}\n",
        escape_toml(&lineage.cursor_path),
        lineage.entries.len()
    );
    for entry in &lineage.entries {
        output.push_str(&format!(
            "\n[[entry]]\n\
             sequence = {}\n\
             previous_hash = \"{}\"\n\
             current_hash = \"{}\"\n\
             after_frame_id = \"{}\"\n\
             next_frame_id = \"{}\"\n",
            entry.sequence,
            entry.previous_hash,
            entry.current_hash,
            escape_toml(&entry.after_frame_id),
            escape_toml(&entry.next_frame_id),
        ));
    }
    output
}

fn load_cursor_lineage(path: &Path, expected_cursor_path: &str) -> Result<CursorLineage, String> {
    let source = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read cursor lineage `{}`: {error}",
            path.display()
        )
    })?;
    parse_cursor_lineage(&source, expected_cursor_path)
        .map_err(|error| format!("invalid cursor lineage `{}`: {error}", path.display()))
}

fn parse_cursor_lineage(source: &str, expected_cursor_path: &str) -> Result<CursorLineage, String> {
    let mut header = BTreeMap::new();
    let mut entries = Vec::new();
    let mut entry = None::<BTreeMap<String, String>>;
    for (index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[entry]]" {
            if let Some(fields) = entry.take() {
                entries.push(parse_lineage_entry(fields)?);
            }
            entry = Some(BTreeMap::new());
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("invalid line {}", index + 1))?;
        let fields = entry.as_mut().unwrap_or(&mut header);
        if fields
            .insert(key.trim().to_owned(), parse_value(value.trim())?)
            .is_some()
        {
            return Err(format!("duplicate field `{}`", key.trim()));
        }
    }
    if let Some(fields) = entry {
        entries.push(parse_lineage_entry(fields)?);
    }
    require_exact_keys(
        &header,
        &["protocol", "cursor_path", "entry_limit", "entry_count"],
    )?;
    require_value(&header, "protocol", LINEAGE_PROTOCOL)?;
    require_value(&header, "cursor_path", expected_cursor_path)?;
    require_value(&header, "entry_limit", &LINEAGE_LIMIT.to_string())?;
    let declared_count = field(&header, "entry_count")?
        .parse::<usize>()
        .map_err(|_| "entry_count must be an unsigned integer".to_owned())?;
    if entries.len() != declared_count || entries.len() > LINEAGE_LIMIT {
        return Err(format!(
            "entry count {} does not match declared {declared_count} or limit {LINEAGE_LIMIT}",
            entries.len()
        ));
    }
    validate_lineage_entries(&entries)?;
    Ok(CursorLineage {
        cursor_path: expected_cursor_path.to_owned(),
        entries,
    })
}

fn parse_lineage_entry(fields: BTreeMap<String, String>) -> Result<CursorLineageEntry, String> {
    require_exact_keys(
        &fields,
        &[
            "sequence",
            "previous_hash",
            "current_hash",
            "after_frame_id",
            "next_frame_id",
        ],
    )?;
    Ok(CursorLineageEntry {
        sequence: field(&fields, "sequence")?
            .parse::<u64>()
            .map_err(|_| "sequence must be an unsigned integer".to_owned())?,
        previous_hash: field(&fields, "previous_hash")?.to_owned(),
        current_hash: field(&fields, "current_hash")?.to_owned(),
        after_frame_id: field(&fields, "after_frame_id")?.to_owned(),
        next_frame_id: field(&fields, "next_frame_id")?.to_owned(),
    })
}

fn validate_lineage_entries(entries: &[CursorLineageEntry]) -> Result<(), String> {
    for (index, entry) in entries.iter().enumerate() {
        if entry.previous_hash != "none" && !is_fnv1a64_hex(&entry.previous_hash) {
            return Err(format!("entry {index} has invalid previous_hash"));
        }
        if !is_fnv1a64_hex(&entry.current_hash) {
            return Err(format!("entry {index} has invalid current_hash"));
        }
        if let Some(previous) = index.checked_sub(1).and_then(|i| entries.get(i)) {
            if entry.sequence != previous.sequence + 1
                || entry.previous_hash != previous.current_hash
            {
                return Err(format!("entry {index} breaks the cursor hash chain"));
            }
        }
    }
    Ok(())
}

fn require_exact_keys(fields: &BTreeMap<String, String>, expected: &[&str]) -> Result<(), String> {
    if fields.len() != expected.len() || expected.iter().any(|key| !fields.contains_key(*key)) {
        return Err("lineage fields do not match the protocol".to_owned());
    }
    Ok(())
}

fn require_value(
    fields: &BTreeMap<String, String>,
    key: &str,
    expected: &str,
) -> Result<(), String> {
    let actual = field(fields, key)?;
    if actual != expected {
        return Err(format!("{key} must be `{expected}`, found `{actual}`"));
    }
    Ok(())
}

fn field<'a>(fields: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
    fields
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing lineage field `{key}`"))
}

fn parse_value(value: &str) -> Result<String, String> {
    if !value.starts_with('"') {
        return Ok(value.to_owned());
    }
    if !value.ends_with('"') || value.len() < 2 {
        return Err("unterminated lineage string".to_owned());
    }
    Ok(value[1..value.len() - 1]
        .replace("\\\"", "\"")
        .replace("\\\\", "\\"))
}

fn is_fnv1a64_hex(value: &str) -> bool {
    value.len() == 18
        && value.starts_with("0x")
        && value[2..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nsdb-cursor-lineage-{label}-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn records_and_bounds_a_hash_checked_cursor_lineage() {
        let root = temp_dir("bounded");
        let cursor = root.join("cursor.toml");
        let mut previous = None::<String>;
        for index in 0..12 {
            let current = format!("cursor-{index}");
            record_cursor_lineage(
                &cursor,
                previous.as_deref(),
                &current,
                &format!("frame-{index}"),
                &format!("frame-{}", index + 1),
            )
            .unwrap();
            previous = Some(current);
        }

        let lineage = load_cursor_lineage(
            &cursor_lineage_path(&cursor).unwrap(),
            &cursor.display().to_string(),
        )
        .unwrap();
        assert_eq!(lineage.entries.len(), LINEAGE_LIMIT);
        assert_eq!(lineage.entries.first().unwrap().sequence, 4);
        assert_eq!(lineage.entries.last().unwrap().sequence, 11);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn damaged_lineage_is_not_replaced() {
        let root = temp_dir("damaged");
        let cursor = root.join("cursor.toml");
        let sidecar = cursor_lineage_path(&cursor).unwrap();
        fs::write(&sidecar, "protocol = \"damaged\"\n").unwrap();

        let error =
            record_cursor_lineage(&cursor, None, "cursor-0", "frame-0", "frame-1").unwrap_err();

        assert!(error.contains("invalid cursor lineage"));
        assert_eq!(
            fs::read_to_string(sidecar).unwrap(),
            "protocol = \"damaged\"\n"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn repair_archives_invalid_lineage_and_then_becomes_idempotent() {
        let root = temp_dir("repair");
        let manifest = root.join("nuis.build.manifest.toml");
        let cursor = root.join(CURSOR_FILE_NAME);
        fs::write(&manifest, "manifest = true\n").unwrap();
        fs::write(
            &cursor,
            format!(
                "protocol = \"nsdb-yir-replay-cursor-record-v1\"\n\
                 transcript_contract = \"nsdb-yir-replay-transcript-v1\"\n\
                 source_contract = \"nsdb-payload-execution-replay-plan-v1\"\n\
                 manifest = \"{}\"\nstatus = \"resume-ready\"\n\
                 after_frame_id = \"frame-0\"\nnext_frame_index = 1\n\
                 next_frame_id = \"frame-1\"\n",
                manifest.display()
            ),
        )
        .unwrap();
        let lineage = cursor_lineage_path(&cursor).unwrap();
        fs::write(&lineage, "protocol = \"damaged\"\n").unwrap();

        let repaired = repair_cursor_lineage(&root, &manifest).unwrap();
        assert_eq!(repaired.status, "lineage-rebuilt");
        assert!(repaired.mutated);
        assert!(repaired.lineage_mutated);
        assert!(repaired.repair_journal_mutated);
        assert_eq!(repaired.entry_count, 1);
        assert!(repaired
            .archived_path
            .as_deref()
            .is_some_and(|path| Path::new(path).exists()));
        let journal_path = root.join(repair_journal::FILE_NAME);
        let journal_before = fs::read_to_string(&journal_path).unwrap();
        assert!(repair_journal::journal_is_valid(&journal_path, &lineage));
        assert!(journal_before.contains("status = \"lineage-rebuilt\""));
        assert!(journal_before.contains(&format!("rebuilt_hash = \"{}\"", repaired.latest_hash)));

        let repeated = repair_cursor_lineage(&root, &manifest).unwrap();
        assert_eq!(repeated.status, "already-ready");
        assert!(!repeated.mutated);
        assert!(!repeated.lineage_mutated);
        assert!(!repeated.repair_journal_mutated);
        assert_eq!(repeated.latest_hash, repaired.latest_hash);
        assert_eq!(fs::read_to_string(&journal_path).unwrap(), journal_before);

        let healthy_lineage = fs::read_to_string(&lineage).unwrap();
        fs::write(&journal_path, "protocol = \"journal-only-damage\"\n").unwrap();
        let journal_only = repair_cursor_lineage(&root, &manifest).unwrap();
        assert_eq!(journal_only.status, "repair-history-recovered");
        assert!(journal_only.mutated);
        assert!(!journal_only.lineage_mutated);
        assert!(journal_only.repair_journal_mutated);
        assert_eq!(fs::read_to_string(&lineage).unwrap(), healthy_lineage);
        let journal_only_source = fs::read_to_string(&journal_path).unwrap();
        assert!(journal_only_source.contains("status = \"repair-history-recovered\""));
        assert!(repair_journal::journal_is_valid(&journal_path, &lineage));

        fs::write(&lineage, "protocol = \"damaged-again\"\n").unwrap();
        fs::write(&journal_path, "protocol = \"damaged-journal\"\n").unwrap();
        let recovered = repair_cursor_lineage(&root, &manifest).unwrap();
        assert_eq!(recovered.status, "lineage-rebuilt");
        assert!(recovered
            .archived_repair_journal_path
            .as_deref()
            .is_some_and(|path| Path::new(path).exists()));
        assert!(repair_journal::journal_is_valid(&journal_path, &lineage));

        let damaged_lineage = "protocol = \"must-remain\"\n";
        let damaged_journal = "protocol = \"cannot-archive\"\n";
        fs::write(&lineage, damaged_lineage).unwrap();
        fs::write(&journal_path, damaged_journal).unwrap();
        let hash = fnv1a64_hex(damaged_journal.as_bytes())
            .trim_start_matches("0x")
            .to_owned();
        let stem = repair_journal::FILE_NAME.trim_end_matches(".toml");
        for suffix in 0..16 {
            let suffix = if suffix == 0 {
                String::new()
            } else {
                format!("-{suffix}")
            };
            fs::write(
                root.join(format!("{stem}.invalid-{hash}{suffix}.toml")),
                "reserved\n",
            )
            .unwrap();
        }
        let error = repair_cursor_lineage(&root, &manifest).unwrap_err();
        assert!(error.contains("failed to reserve an archive path"));
        assert_eq!(fs::read_to_string(&lineage).unwrap(), damaged_lineage);
        fs::remove_dir_all(root).unwrap();
    }
}
