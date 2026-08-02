use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::artifact_nsdb_handoff::read_persisted_nsdb_handoff;

const CURSOR_FILE_NAME: &str = "nuis.nsdb.replay-cursor.toml";
const CURSOR_SELECTION_FILE_NAME: &str = "nuis.debugger.cursor-selection.toml";
const CURSOR_SELECTION_PROTOCOL: &str = "nuis-debugger-cursor-selection-v1";
const CURSOR_PROTOCOL: &str = "nsdb-yir-replay-cursor-record-v2";
const TRANSCRIPT_PROTOCOL: &str = "nsdb-yir-replay-transcript-v1";
const SOURCE_CONTRACT: &str = "nsdb-payload-execution-replay-plan-v1";
static CURSOR_SELECTION_TEMP_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) struct DebuggerCursorHandoffMirror {
    pub(crate) contract: &'static str,
    pub(crate) path: String,
    pub(crate) ready: bool,
    pub(crate) status: &'static str,
    pub(crate) next_command: Option<String>,
}

pub(crate) fn read_debugger_cursor_handoff(
    output_dir: &Path,
    manifest: &Path,
) -> DebuggerCursorHandoffMirror {
    let path = match selected_cursor_path(output_dir) {
        Ok(path) => path,
        Err(status) => {
            return DebuggerCursorHandoffMirror {
                contract: "nuis-debugger-cursor-handoff-v1",
                path: output_dir
                    .join(CURSOR_SELECTION_FILE_NAME)
                    .display()
                    .to_string(),
                ready: false,
                status,
                next_command: None,
            };
        }
    };
    let path_text = path.display().to_string();
    let Ok(source) = fs::read_to_string(&path) else {
        return DebuggerCursorHandoffMirror {
            contract: "nuis-debugger-cursor-handoff-v1",
            path: path_text,
            ready: false,
            status: "cursor-unavailable",
            next_command: None,
        };
    };
    let handoff = read_persisted_nsdb_handoff(Some(output_dir));
    let expected_proof_hash = handoff.final_image_binding_proof_hash();
    let dispatch_identity = handoff.provider_dispatch_identity();
    let ready = field(&source, "protocol").as_deref() == Some(CURSOR_PROTOCOL)
        && field(&source, "transcript_contract").as_deref() == Some(TRANSCRIPT_PROTOCOL)
        && field(&source, "source_contract").as_deref() == Some(SOURCE_CONTRACT)
        && field(&source, "identity_contract").as_deref() == Some("nsdb-yir-replay-identity-v1")
        && matches!(
            handoff.final_image_binding_proof_status(),
            "verified" | "verified-empty"
        )
        && expected_proof_hash.is_some_and(|expected| {
            field(&source, "final_image_binding_proof_hash").as_deref() == Some(expected)
        })
        && field(&source, "provider_dispatch_authority_contract").as_deref()
            == Some(dispatch_identity.contract.as_str())
        && field(&source, "provider_dispatch_table_hash").as_deref()
            == Some(dispatch_identity.table_hash.as_str())
        && field(&source, "provider_dispatch_selected_set_hash").as_deref()
            == Some(dispatch_identity.selected_set_hash.as_str())
        && field(&source, "provider_dispatch_identity_hash").as_deref()
            == Some(dispatch_identity.identity_hash.as_str())
        && field(&source, "status").as_deref() == Some("resume-ready")
        && field(&source, "after_frame_id").is_some_and(|value| !value.is_empty())
        && field(&source, "next_frame_id").is_some_and(|value| !value.is_empty())
        && field(&source, "next_frame_index")
            .and_then(|value| value.parse::<usize>().ok())
            .is_some()
        && field(&source, "manifest")
            .is_some_and(|recorded| same_manifest(Path::new(&recorded), manifest));
    let next_command = ready.then(|| format!("nuis debug-resume {} --json", output_dir.display()));
    DebuggerCursorHandoffMirror {
        contract: "nuis-debugger-cursor-handoff-v1",
        path: path_text,
        ready,
        status: if ready {
            "cursor-resume-ready"
        } else {
            "cursor-invalid"
        },
        next_command,
    }
}

pub(crate) fn resolve_debugger_cursor_output(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    std::env::current_dir()
        .map(|current| current.join(path))
        .map_err(|error| format!("failed to resolve debugger cursor output: {error}"))
}

pub(crate) fn persist_debugger_cursor_selection(
    output_dir: &Path,
    cursor_path: &Path,
) -> Result<(), String> {
    if !cursor_path.is_file() {
        return Err(format!(
            "debugger cursor output `{}` was not materialized",
            cursor_path.display()
        ));
    }
    let selection_path =
        resolve_debugger_cursor_output(&output_dir.join(CURSOR_SELECTION_FILE_NAME))?;
    if cursor_path == selection_path {
        return Err("debugger cursor output cannot replace its selection record".to_owned());
    }
    let content = format!(
        "protocol = \"{CURSOR_SELECTION_PROTOCOL}\"\nstatus = \"selected\"\ncursor_path = \"{}\"\n",
        escape(&cursor_path.display().to_string())
    );
    persist_selection_atomically(&selection_path, content.as_bytes())
}

fn selected_cursor_path(output_dir: &Path) -> Result<PathBuf, &'static str> {
    let selection_path = output_dir.join(CURSOR_SELECTION_FILE_NAME);
    let source = match fs::read_to_string(&selection_path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(output_dir.join(CURSOR_FILE_NAME));
        }
        Err(_) => return Err("cursor-selection-unreadable"),
    };
    if field(&source, "protocol").as_deref() != Some(CURSOR_SELECTION_PROTOCOL)
        || field(&source, "status").as_deref() != Some("selected")
    {
        return Err("cursor-selection-invalid");
    }
    let Some(path) = field(&source, "cursor_path").filter(|path| !path.is_empty()) else {
        return Err("cursor-selection-invalid");
    };
    let path = PathBuf::from(path);
    Ok(if path.is_absolute() {
        path
    } else {
        output_dir.join(path)
    })
}

fn persist_selection_atomically(path: &Path, content: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "debugger cursor selection path has no file name".to_owned())?;
    for _ in 0..16 {
        let id = CURSOR_SELECTION_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(".{name}.tmp-{}-{id}", std::process::id()));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("failed to create cursor selection: {error}")),
        };
        let result = (|| {
            file.write_all(content)
                .map_err(|error| format!("failed to write cursor selection: {error}"))?;
            file.sync_all()
                .map_err(|error| format!("failed to sync cursor selection: {error}"))?;
            drop(file);
            fs::rename(&temporary, path)
                .map_err(|error| format!("failed to install cursor selection: {error}"))?;
            sync_directory(parent)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        return result;
    }
    Err("failed to reserve cursor selection temporary file".to_owned())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        std::fs::File::open(path)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| format!("failed to sync cursor selection directory: {error}"))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
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

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn same_manifest(recorded: &Path, expected: &Path) -> bool {
    match (recorded.canonicalize(), expected.canonicalize()) {
        (Ok(recorded), Ok(expected)) => recorded == expected,
        _ => recorded == expected,
    }
}

#[cfg(test)]
mod tests {
    use super::{persist_debugger_cursor_selection, read_debugger_cursor_handoff};
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn mirrors_ready_cursor_without_exposing_nsdb_types() {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "nuis-debugger-cursor-mirror-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create cursor mirror test directory");
        let manifest = root.join("nuis.build.manifest.toml");
        fs::write(&manifest, "manifest = true\n").expect("write manifest");
        fs::write(
            root.join("nuis.nsdb.replay-cursor.toml"),
            format!(
                "protocol = \"nsdb-yir-replay-cursor-record-v2\"\n\
                 transcript_contract = \"nsdb-yir-replay-transcript-v1\"\n\
                 source_contract = \"nsdb-payload-execution-replay-plan-v1\"\n\
                 identity_contract = \"nsdb-yir-replay-identity-v1\"\n\
                 final_image_binding_proof_hash = \"fnv1a64:981b10a68f4e3dd7\"\n\
                 provider_dispatch_authority_contract = \"nuis-provider-completion-dispatch-authority-v1\"\n\
                 provider_dispatch_table_hash = \"none\"\n\
                 provider_dispatch_selected_set_hash = \"none\"\n\
                 provider_dispatch_identity_hash = \"none\"\n\
                 manifest = \"{}\"\n\
                 status = \"resume-ready\"\n\
                 after_frame_id = \"frame-0\"\n\
                 next_frame_index = 1\n\
                 next_frame_id = \"frame-1\"\n",
                manifest.display()
            ),
        )
        .expect("write cursor");
        fs::write(
            root.join("nuis.nsdb.payload-execution-handoff.toml"),
            "final_image_binding_proof_contract = \"nuis-final-image-binding-proof-v1\"\n\
             final_image_metadata_binding_count = 0\n\
             final_image_metadata_binding_table_hash = \"0xcbf29ce484222325\"\n\
             final_image_metadata_binding_validation_status = \"not-applicable\"\n\
             final_image_selected_provider_bundle_set_contract = \"\"\n\
             final_image_selected_provider_bundle_count = 0\n\
             final_image_selected_provider_bundle_set_hash = \"\"\n\
             final_image_binding_proof_hash = \"fnv1a64:981b10a68f4e3dd7\"\n",
        )
        .expect("write handoff proof");

        let mirror = read_debugger_cursor_handoff(&root, &manifest);
        assert_eq!(mirror.contract, "nuis-debugger-cursor-handoff-v1");
        assert!(mirror.ready);
        assert_eq!(mirror.status, "cursor-resume-ready");
        assert!(mirror
            .next_command
            .as_deref()
            .is_some_and(|command| command.starts_with("nuis debug-resume ")));

        let cursor_path = root.join("nuis.nsdb.replay-cursor.toml");
        let selected_cursor_path = root.join("selected-request.cursor.toml");
        fs::rename(&cursor_path, &selected_cursor_path).expect("move selected cursor");
        persist_debugger_cursor_selection(&root, &selected_cursor_path)
            .expect("persist cursor selection");
        let selected = read_debugger_cursor_handoff(&root, &manifest);
        assert!(selected.ready);
        assert_eq!(selected.path, selected_cursor_path.display().to_string());

        let selected_source = fs::read_to_string(&selected_cursor_path).unwrap();
        let drifted =
            selected_source.replace("fnv1a64:981b10a68f4e3dd7", "fnv1a64:fedcba9876543210");
        fs::write(&selected_cursor_path, drifted).unwrap();
        let rejected = read_debugger_cursor_handoff(&root, &manifest);
        assert!(!rejected.ready);
        assert_eq!(rejected.status, "cursor-invalid");

        fs::write(&selected_cursor_path, &selected_source).unwrap();
        fs::write(root.join("nuis.nsdb.replay-cursor.toml"), selected_source).unwrap();
        fs::write(
            root.join(super::CURSOR_SELECTION_FILE_NAME),
            "protocol = \"invalid\"\nstatus = \"selected\"\n",
        )
        .unwrap();
        let malformed_selection = read_debugger_cursor_handoff(&root, &manifest);
        assert!(!malformed_selection.ready);
        assert_eq!(malformed_selection.status, "cursor-selection-invalid");
        fs::remove_dir_all(root).expect("remove cursor mirror test directory");
    }
}
