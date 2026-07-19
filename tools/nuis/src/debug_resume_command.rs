use crate::artifact_nsdb_replay_cursor::read_debugger_cursor_handoff;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) fn handle_debug_resume(input: PathBuf, json: bool) -> Result<(), String> {
    let (output_dir, manifest) = resolve_debug_resume_input(&input)?;
    let cursor = read_debugger_cursor_handoff(&output_dir, &manifest);
    if !cursor.ready {
        return Err(format!(
            "debug-resume rejected `{}`: {} at `{}`",
            input.display(),
            cursor.status,
            cursor.path
        ));
    }

    let mut command = Command::new(resolve_nsdb_program());
    command
        .arg("replay")
        .arg(&output_dir)
        .arg("--resume-cursor")
        .arg(&cursor.path);
    if json {
        command.arg("--json");
    }
    let status = command
        .status()
        .map_err(|error| format!("failed to start Nsdb debug resume: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Nsdb debug resume failed with status {}",
            status.code().unwrap_or(1)
        ))
    }
}

fn resolve_debug_resume_input(input: &Path) -> Result<(PathBuf, PathBuf), String> {
    let (output_dir, manifest) = if input.is_dir() {
        (input.to_path_buf(), input.join("nuis.build.manifest.toml"))
    } else if input.file_name().and_then(|name| name.to_str()) == Some("nuis.build.manifest.toml") {
        let output_dir = input
            .parent()
            .ok_or_else(|| "debug-resume manifest has no output directory".to_owned())?
            .to_path_buf();
        (output_dir, input.to_path_buf())
    } else {
        return Err(format!(
            "debug-resume expected an artifact output directory or `nuis.build.manifest.toml`, found `{}`",
            input.display()
        ));
    };
    if !manifest.is_file() {
        return Err(format!(
            "debug-resume output `{}` does not contain `nuis.build.manifest.toml`",
            output_dir.display()
        ));
    }
    Ok((output_dir, manifest))
}

fn resolve_nsdb_program() -> PathBuf {
    if let Some(program) = std::env::var_os("NUIS_NSDB_BIN") {
        return PathBuf::from(program);
    }
    if let Ok(current) = std::env::current_exe() {
        let sibling = current.with_file_name(format!("nsdb{}", std::env::consts::EXE_SUFFIX));
        if sibling.is_file() {
            return sibling;
        }
    }
    PathBuf::from(format!("nsdb{}", std::env::consts::EXE_SUFFIX))
}

#[cfg(test)]
mod tests {
    use super::resolve_debug_resume_input;
    use std::fs;

    #[test]
    fn resolves_output_directory_and_manifest_inputs() {
        let root =
            std::env::temp_dir().join(format!("nuis-debug-resume-input-{}", std::process::id()));
        fs::create_dir_all(&root).expect("create debug resume test directory");
        let manifest = root.join("nuis.build.manifest.toml");
        fs::write(&manifest, "manifest = true\n").expect("write manifest");

        assert_eq!(
            resolve_debug_resume_input(&root).expect("resolve directory"),
            (root.clone(), manifest.clone())
        );
        assert_eq!(
            resolve_debug_resume_input(&manifest).expect("resolve manifest"),
            (root.clone(), manifest)
        );
        fs::remove_dir_all(root).expect("remove debug resume test directory");
    }
}
