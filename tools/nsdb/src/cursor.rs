use std::{collections::BTreeMap, fs, path::Path};

use crate::transcript::{NsdbReplayControl, NsdbReplayTranscript};

const CURSOR_PROTOCOL: &str = "nsdb-yir-replay-cursor-record-v1";
const TRANSCRIPT_PROTOCOL: &str = "nsdb-yir-replay-transcript-v1";
const SOURCE_CONTRACT: &str = "nsdb-payload-execution-replay-plan-v1";

pub(crate) fn persist_replay_cursor(
    output: &Path,
    manifest: &Path,
    transcript: &NsdbReplayTranscript,
) -> Result<(), String> {
    if !transcript.ready || !transcript.resume_cursor_ready {
        return Err(format!(
            "cannot persist replay cursor: transcript cursor status is `{}`",
            transcript.resume_cursor_status
        ));
    }
    let after = transcript
        .resume_after_frame_id
        .as_deref()
        .ok_or_else(|| "cannot persist replay cursor without stopped frame".to_owned())?;
    let next = transcript
        .resume_next_frame_id
        .as_deref()
        .ok_or_else(|| "cannot persist replay cursor without next frame".to_owned())?;
    let next_index = transcript
        .resume_next_frame_index
        .ok_or_else(|| "cannot persist replay cursor without next frame index".to_owned())?;
    let content = format!(
        "protocol = \"{CURSOR_PROTOCOL}\"\n\
         transcript_contract = \"{}\"\n\
         source_contract = \"{}\"\n\
         manifest = \"{}\"\n\
         status = \"resume-ready\"\n\
         after_frame_id = \"{}\"\n\
         next_frame_index = {}\n\
         next_frame_id = \"{}\"\n",
        transcript.protocol,
        transcript.source_contract,
        escape_toml(&manifest.display().to_string()),
        escape_toml(after),
        next_index,
        escape_toml(next),
    );
    fs::write(output, content).map_err(|error| {
        format!(
            "failed to persist replay cursor `{}`: {error}",
            output.display()
        )
    })
}

pub(crate) fn load_replay_cursor(
    input: &Path,
    manifest: &Path,
) -> Result<NsdbReplayControl, String> {
    let source = fs::read_to_string(input).map_err(|error| {
        format!(
            "failed to read replay cursor `{}`: {error}",
            input.display()
        )
    })?;
    let fields = parse_cursor_fields(&source)?;
    require_field(&fields, "protocol", CURSOR_PROTOCOL)?;
    require_field(&fields, "transcript_contract", TRANSCRIPT_PROTOCOL)?;
    require_field(&fields, "source_contract", SOURCE_CONTRACT)?;
    require_field(&fields, "status", "resume-ready")?;

    let recorded_manifest = field(&fields, "manifest")?;
    if !same_manifest(Path::new(recorded_manifest), manifest) {
        return Err(format!(
            "replay cursor manifest `{recorded_manifest}` does not match `{}`",
            manifest.display()
        ));
    }
    field(&fields, "next_frame_index")?
        .parse::<usize>()
        .map_err(|_| "replay cursor `next_frame_index` must be an unsigned integer".to_owned())?;

    Ok(NsdbReplayControl {
        resume_after_frame_id: Some(field(&fields, "after_frame_id")?.to_owned()),
        resume_next_frame_id: Some(field(&fields, "next_frame_id")?.to_owned()),
        ..NsdbReplayControl::default()
    })
}

fn parse_cursor_fields(source: &str) -> Result<BTreeMap<String, String>, String> {
    let mut fields = BTreeMap::new();
    for (index, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("invalid replay cursor line {}", index + 1))?;
        let key = key.trim();
        if !matches!(
            key,
            "protocol"
                | "transcript_contract"
                | "source_contract"
                | "manifest"
                | "status"
                | "after_frame_id"
                | "next_frame_index"
                | "next_frame_id"
        ) {
            return Err(format!("unknown replay cursor field `{key}`"));
        }
        let value = parse_cursor_value(value.trim())?;
        if fields.insert(key.to_owned(), value).is_some() {
            return Err(format!("duplicate replay cursor field `{key}`"));
        }
    }
    Ok(fields)
}

fn parse_cursor_value(value: &str) -> Result<String, String> {
    if !value.starts_with('"') {
        return Ok(value.to_owned());
    }
    if !value.ends_with('"') || value.len() < 2 {
        return Err("unterminated replay cursor string".to_owned());
    }
    let mut output = String::new();
    let mut escaped = false;
    for character in value[1..value.len() - 1].chars() {
        if escaped {
            match character {
                '\\' | '"' => output.push(character),
                _ => return Err(format!("unsupported replay cursor escape `\\{character}`")),
            }
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    if escaped {
        return Err("unterminated replay cursor escape".to_owned());
    }
    Ok(output)
}

fn field<'a>(fields: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
    fields
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("replay cursor is missing `{key}`"))
}

fn require_field(
    fields: &BTreeMap<String, String>,
    key: &str,
    expected: &str,
) -> Result<(), String> {
    let actual = field(fields, key)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "replay cursor `{key}` must be `{expected}`, found `{actual}`"
        ))
    }
}

fn same_manifest(recorded: &Path, requested: &Path) -> bool {
    match (recorded.canonicalize(), requested.canonicalize()) {
        (Ok(recorded), Ok(requested)) => recorded == requested,
        _ => recorded == requested,
    }
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::load_replay_cursor;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("nsdb-cursor-{label}-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).expect("create cursor test directory");
        path
    }

    fn cursor_source(manifest: &Path, extra: &str) -> String {
        format!(
            "protocol = \"nsdb-yir-replay-cursor-record-v1\"\n\
             transcript_contract = \"nsdb-yir-replay-transcript-v1\"\n\
             source_contract = \"nsdb-payload-execution-replay-plan-v1\"\n\
             manifest = \"{}\"\n\
             status = \"resume-ready\"\n\
             after_frame_id = \"frame-0\"\n\
             next_frame_index = 1\n\
             next_frame_id = \"frame-1\"\n\
             {extra}",
            manifest.display()
        )
    }

    #[test]
    fn rejects_cursor_for_another_manifest() {
        let root = temp_dir("manifest-mismatch");
        let recorded = root.join("recorded.toml");
        let requested = root.join("requested.toml");
        let cursor = root.join("cursor.toml");
        fs::write(&recorded, "manifest = true\n").expect("write recorded manifest");
        fs::write(&requested, "manifest = true\n").expect("write requested manifest");
        fs::write(&cursor, cursor_source(&recorded, "")).expect("write cursor");

        let error = load_replay_cursor(&cursor, &requested).unwrap_err();
        assert!(error.contains("does not match"));
        fs::remove_dir_all(root).expect("remove cursor test directory");
    }

    #[test]
    fn rejects_unknown_cursor_fields() {
        let root = temp_dir("unknown-field");
        let manifest = root.join("manifest.toml");
        let cursor = root.join("cursor.toml");
        fs::write(&manifest, "manifest = true\n").expect("write manifest");
        fs::write(&cursor, cursor_source(&manifest, "surprise = \"field\"\n"))
            .expect("write cursor");

        let error = load_replay_cursor(&cursor, &manifest).unwrap_err();
        assert!(error.contains("unknown replay cursor field"));
        fs::remove_dir_all(root).expect("remove cursor test directory");
    }
}
