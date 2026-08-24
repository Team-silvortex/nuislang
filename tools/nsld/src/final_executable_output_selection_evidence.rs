use crate::{
    final_executable_output_selection::validate_final_output_selection_report,
    json_fields::json_string_field,
    json_final_output_selection::relocatable_final_output_selection_json_field,
    reports::NsldFinalOutputSelectionReport,
};
use std::{
    fs::{self, File, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) const FINAL_OUTPUT_SELECTION_EVIDENCE_FILE_CONTRACT: &str =
    "nuis-nsld-final-output-selection-evidence-file-v1";
pub(crate) const FINAL_OUTPUT_SELECTION_EVIDENCE_FILE: &str =
    "nuis.nsld.final-output-selection-evidence.json";

static EVIDENCE_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn final_output_selection_evidence_path(output_dir: &Path) -> PathBuf {
    output_dir.join(FINAL_OUTPUT_SELECTION_EVIDENCE_FILE)
}

pub(crate) fn persist_final_output_selection_evidence(
    output_dir: &Path,
    report: &NsldFinalOutputSelectionReport,
) -> Result<PathBuf, String> {
    if !report.explicit_request {
        return Err(
            "final-output selection evidence persistence requires an explicit policy".to_owned(),
        );
    }
    validate_final_output_selection_report(report)?;
    let source = render_final_output_selection_evidence(report)?;
    let path = final_output_selection_evidence_path(output_dir);
    atomic_write_evidence(&path, source.as_bytes())?;
    Ok(path)
}

pub(crate) fn render_final_output_selection_evidence(
    report: &NsldFinalOutputSelectionReport,
) -> Result<String, String> {
    validate_final_output_selection_report(report)?;
    Ok(format!(
        "{{{},{}}}\n",
        json_string_field("contract", FINAL_OUTPUT_SELECTION_EVIDENCE_FILE_CONTRACT),
        relocatable_final_output_selection_json_field(report)
    ))
}

fn atomic_write_evidence(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create final-output selection evidence directory `{}`: {error}",
            parent.display()
        )
    })?;
    let sequence = EVIDENCE_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_file_name(format!(
        ".{FINAL_OUTPUT_SELECTION_EVIDENCE_FILE}.{}-{sequence}.tmp",
        std::process::id()
    ));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        let mut file = options.open(&temp).map_err(|error| {
            format!(
                "failed to create temporary final-output selection evidence `{}`: {error}",
                temp.display()
            )
        })?;
        file.write_all(bytes).map_err(|error| {
            format!(
                "failed to write temporary final-output selection evidence `{}`: {error}",
                temp.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "failed to sync temporary final-output selection evidence `{}`: {error}",
                temp.display()
            )
        })?;
        drop(file);
        fs::rename(&temp, path).map_err(|error| {
            format!(
                "failed to atomically install final-output selection evidence `{}`: {error}",
                path.display()
            )
        })?;
        sync_directory(parent)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            format!(
                "failed to sync final-output selection evidence directory `{}`: {error}",
                path.display()
            )
        })
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
#[path = "final_executable_output_selection_evidence_tests.rs"]
mod tests;
