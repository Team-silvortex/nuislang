use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{cursor::persist_validated_content_atomically, provider_sample_payload::fnv1a64_hex};

pub(super) const FILE_NAME: &str = "nuis.nsdb.replay-cursor.lineage-repairs.toml";
const PROTOCOL: &str = "nsdb-yir-replay-cursor-lineage-repair-journal-v2";
const ENTRY_LIMIT: usize = 8;

pub(super) struct RepairJournalPreflight {
    pub(super) path: PathBuf,
    pub(super) archived_path: Option<PathBuf>,
    pub(super) archived_hash: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Entry {
    sequence: u64,
    status: String,
    lineage_mutated: bool,
    repair_journal_mutated: bool,
    archived_path: String,
    archived_hash: String,
    archived_repair_journal_path: String,
    archived_repair_journal_hash: String,
    rebuilt_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Journal {
    lineage_path: String,
    entries: Vec<Entry>,
}

pub(super) fn preflight(
    output_dir: &Path,
    lineage_path: &Path,
) -> Result<RepairJournalPreflight, String> {
    let path = output_dir.join(FILE_NAME);
    let lineage_path_text = lineage_path.display().to_string();
    if !path.exists() || load(&path, &lineage_path_text).is_ok() {
        return Ok(RepairJournalPreflight {
            path,
            archived_path: None,
            archived_hash: None,
        });
    }
    let bytes = fs::read(&path).map_err(|error| {
        format!(
            "failed to read invalid cursor lineage repair journal `{}`: {error}",
            path.display()
        )
    })?;
    let archived_hash = fnv1a64_hex(&bytes);
    let archived_path = archive_invalid(&path, &archived_hash)?;
    Ok(RepairJournalPreflight {
        path,
        archived_path: Some(archived_path),
        archived_hash: Some(archived_hash),
    })
}

pub(super) fn record(
    preflight: &RepairJournalPreflight,
    lineage_path: &Path,
    status: &str,
    lineage_mutated: bool,
    archived_lineage_path: Option<&Path>,
    rebuilt_hash: &str,
) -> Result<(), String> {
    let lineage_path_text = lineage_path.display().to_string();
    let mut journal = if preflight.path.exists() {
        load(&preflight.path, &lineage_path_text)?
    } else {
        Journal {
            lineage_path: lineage_path_text.clone(),
            entries: Vec::new(),
        }
    };
    let (archived_path, archived_hash) = path_and_hash(archived_lineage_path)?;
    let archived_repair_journal_path = preflight
        .archived_path
        .as_deref()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let archived_repair_journal_hash = preflight
        .archived_hash
        .clone()
        .unwrap_or_else(|| "none".to_owned());
    let sequence = journal
        .entries
        .last()
        .map(|entry| entry.sequence + 1)
        .unwrap_or(0);
    journal.entries.push(Entry {
        sequence,
        status: status.to_owned(),
        lineage_mutated,
        repair_journal_mutated: true,
        archived_path,
        archived_hash,
        archived_repair_journal_path,
        archived_repair_journal_hash,
        rebuilt_hash: rebuilt_hash.to_owned(),
    });
    if journal.entries.len() > ENTRY_LIMIT {
        journal.entries.drain(..journal.entries.len() - ENTRY_LIMIT);
    }
    let content = render(&journal);
    persist_validated_content_atomically(&preflight.path, &content, |temporary| {
        load(temporary, &lineage_path_text).map(|_| ())
    })
}

fn path_and_hash(path: Option<&Path>) -> Result<(String, String), String> {
    let Some(path) = path else {
        return Ok((String::new(), "none".to_owned()));
    };
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read archived cursor lineage `{}`: {error}",
            path.display()
        )
    })?;
    Ok((path.display().to_string(), fnv1a64_hex(&bytes)))
}

fn render(journal: &Journal) -> String {
    let mut output = format!(
        "protocol = \"{PROTOCOL}\"\nlineage_path = \"{}\"\nentry_limit = {ENTRY_LIMIT}\nentry_count = {}\n",
        escape(&journal.lineage_path),
        journal.entries.len()
    );
    for entry in &journal.entries {
        output.push_str(&format!(
            "\n[[entry]]\nsequence = {}\nstatus = \"{}\"\nlineage_mutated = {}\nrepair_journal_mutated = {}\narchived_path = \"{}\"\narchived_hash = \"{}\"\narchived_repair_journal_path = \"{}\"\narchived_repair_journal_hash = \"{}\"\nrebuilt_hash = \"{}\"\n",
            entry.sequence,
            entry.status,
            entry.lineage_mutated,
            entry.repair_journal_mutated,
            escape(&entry.archived_path),
            entry.archived_hash,
            escape(&entry.archived_repair_journal_path),
            entry.archived_repair_journal_hash,
            entry.rebuilt_hash,
        ));
    }
    output
}

fn load(path: &Path, expected_lineage_path: &str) -> Result<Journal, String> {
    let source = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read repair journal `{}`: {error}",
            path.display()
        )
    })?;
    parse(&source, expected_lineage_path)
        .map_err(|error| format!("invalid cursor lineage repair journal: {error}"))
}

fn parse(source: &str, expected_lineage_path: &str) -> Result<Journal, String> {
    let mut sections = source.split("[[entry]]");
    let header = fields(sections.next().unwrap_or_default())?;
    require(&header, "protocol", PROTOCOL)?;
    require(&header, "lineage_path", expected_lineage_path)?;
    require(&header, "entry_limit", &ENTRY_LIMIT.to_string())?;
    let declared = value(&header, "entry_count")?
        .parse::<usize>()
        .map_err(|_| "entry_count must be unsigned".to_owned())?;
    let entries = sections.map(parse_entry).collect::<Result<Vec<_>, _>>()?;
    if entries.is_empty() || entries.len() != declared || entries.len() > ENTRY_LIMIT {
        return Err("repair journal entry count is invalid".to_owned());
    }
    for (index, entry) in entries.iter().enumerate() {
        if !matches!(
            entry.status.as_str(),
            "lineage-rebuilt" | "repair-history-recovered"
        ) || !entry.repair_journal_mutated
            || !valid_optional_hash(&entry.archived_hash)
            || !valid_optional_hash(&entry.archived_repair_journal_hash)
            || !is_hash(&entry.rebuilt_hash)
        {
            return Err("repair journal entry contract is invalid".to_owned());
        }
        if entry.status == "repair-history-recovered" && entry.lineage_mutated {
            return Err("journal-only recovery cannot mutate lineage".to_owned());
        }
        if let Some(previous) = index.checked_sub(1).and_then(|index| entries.get(index)) {
            if entry.sequence != previous.sequence + 1 {
                return Err("repair journal sequence is not monotonic".to_owned());
            }
        }
    }
    Ok(Journal {
        lineage_path: expected_lineage_path.to_owned(),
        entries,
    })
}

fn parse_entry(source: &str) -> Result<Entry, String> {
    let fields = fields(source)?;
    Ok(Entry {
        sequence: value(&fields, "sequence")?
            .parse::<u64>()
            .map_err(|_| "sequence must be unsigned".to_owned())?,
        status: value(&fields, "status")?.to_owned(),
        lineage_mutated: value(&fields, "lineage_mutated")?
            .parse::<bool>()
            .map_err(|_| "lineage_mutated must be boolean".to_owned())?,
        repair_journal_mutated: value(&fields, "repair_journal_mutated")?
            .parse::<bool>()
            .map_err(|_| "repair_journal_mutated must be boolean".to_owned())?,
        archived_path: value(&fields, "archived_path")?.to_owned(),
        archived_hash: value(&fields, "archived_hash")?.to_owned(),
        archived_repair_journal_path: value(&fields, "archived_repair_journal_path")?.to_owned(),
        archived_repair_journal_hash: value(&fields, "archived_repair_journal_hash")?.to_owned(),
        rebuilt_hash: value(&fields, "rebuilt_hash")?.to_owned(),
    })
}

fn archive_invalid(path: &Path, hash: &str) -> Result<PathBuf, String> {
    let stem = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("repair journal path `{}` has no file name", path.display()))?
        .trim_end_matches(".toml");
    let hash = hash.trim_start_matches("0x");
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
                "failed to archive invalid cursor lineage repair journal `{}`: {error}",
                path.display()
            )
        })?;
        return Ok(archive);
    }
    Err(format!(
        "failed to reserve an archive path for invalid cursor lineage repair journal `{}`",
        path.display()
    ))
}

fn fields(source: &str) -> Result<BTreeMap<String, String>, String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .try_fold(BTreeMap::new(), |mut fields, line| {
            let (key, raw) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid line `{line}`"))?;
            if fields
                .insert(key.trim().to_owned(), parse_value(raw.trim())?)
                .is_some()
            {
                return Err(format!("duplicate field `{}`", key.trim()));
            }
            Ok(fields)
        })
}

fn require(fields: &BTreeMap<String, String>, key: &str, expected: &str) -> Result<(), String> {
    let actual = value(fields, key)?;
    (actual == expected)
        .then_some(())
        .ok_or_else(|| format!("{key} must be `{expected}`"))
}

fn value<'a>(fields: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
    fields
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing field `{key}`"))
}

fn parse_value(value: &str) -> Result<String, String> {
    if !value.starts_with('"') {
        return Ok(value.to_owned());
    }
    if !value.ends_with('"') || value.len() < 2 {
        return Err("unterminated string".to_owned());
    }
    Ok(value[1..value.len() - 1]
        .replace("\\\"", "\"")
        .replace("\\\\", "\\"))
}

fn valid_optional_hash(value: &str) -> bool {
    value == "none" || is_hash(value)
}
fn is_hash(value: &str) -> bool {
    value.len() == 18
        && value.starts_with("0x")
        && value[2..].chars().all(|c| c.is_ascii_hexdigit())
}
fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
pub(super) fn journal_is_valid(path: &Path, lineage_path: &Path) -> bool {
    load(path, &lineage_path.display().to_string()).is_ok()
}
