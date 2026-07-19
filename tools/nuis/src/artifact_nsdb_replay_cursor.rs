use std::{fs, path::Path};

const CURSOR_FILE_NAME: &str = "nuis.nsdb.replay-cursor.toml";
const CURSOR_PROTOCOL: &str = "nsdb-yir-replay-cursor-record-v1";
const TRANSCRIPT_PROTOCOL: &str = "nsdb-yir-replay-transcript-v1";
const SOURCE_CONTRACT: &str = "nsdb-payload-execution-replay-plan-v1";

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
    let path = output_dir.join(CURSOR_FILE_NAME);
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
    let ready = field(&source, "protocol").as_deref() == Some(CURSOR_PROTOCOL)
        && field(&source, "transcript_contract").as_deref() == Some(TRANSCRIPT_PROTOCOL)
        && field(&source, "source_contract").as_deref() == Some(SOURCE_CONTRACT)
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

fn same_manifest(recorded: &Path, expected: &Path) -> bool {
    match (recorded.canonicalize(), expected.canonicalize()) {
        (Ok(recorded), Ok(expected)) => recorded == expected,
        _ => recorded == expected,
    }
}

#[cfg(test)]
mod tests {
    use super::read_debugger_cursor_handoff;
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
                "protocol = \"nsdb-yir-replay-cursor-record-v1\"\n\
                 transcript_contract = \"nsdb-yir-replay-transcript-v1\"\n\
                 source_contract = \"nsdb-payload-execution-replay-plan-v1\"\n\
                 manifest = \"{}\"\n\
                 status = \"resume-ready\"\n\
                 after_frame_id = \"frame-0\"\n\
                 next_frame_index = 1\n\
                 next_frame_id = \"frame-1\"\n",
                manifest.display()
            ),
        )
        .expect("write cursor");

        let mirror = read_debugger_cursor_handoff(&root, &manifest);
        assert_eq!(mirror.contract, "nuis-debugger-cursor-handoff-v1");
        assert!(mirror.ready);
        assert_eq!(mirror.status, "cursor-resume-ready");
        assert!(mirror
            .next_command
            .as_deref()
            .is_some_and(|command| command.starts_with("nuis debug-resume ")));
        fs::remove_dir_all(root).expect("remove cursor mirror test directory");
    }
}
