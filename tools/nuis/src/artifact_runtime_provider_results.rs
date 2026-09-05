use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

#[cfg(unix)]
#[path = "artifact_runtime_provider_ipc.rs"]
mod ipc;

pub(crate) struct PreparedRuntimeProviderResults {
    pub(crate) stream_path: PathBuf,
    pub(crate) source_yir_path: PathBuf,
    pub(crate) target_count: usize,
    output_dir: PathBuf,
}

impl PreparedRuntimeProviderResults {
    pub(crate) fn run_command(&self, command: &mut Command) -> Result<(ExitStatus, usize), String> {
        #[cfg(unix)]
        {
            ipc::run_command(&self.output_dir, command)
        }
        #[cfg(not(unix))]
        {
            let _ = command;
            Err("runtime provider IPC requires a registered host transport".to_owned())
        }
    }

    pub(crate) fn print_text(&self, invocation_count: usize) {
        println!("  runtime_provider_result_targets: {}", self.target_count);
        println!("  runtime_provider_result_invocations: {invocation_count}");
        println!("  runtime_provider_dispatch_trigger: child-yir-node-ipc");
        println!(
            "  runtime_provider_result_source_yir: {}",
            self.source_yir_path.display()
        );
        println!(
            "  runtime_provider_result_stream: {}",
            self.stream_path.display()
        );
    }
}

pub(crate) fn prepare_runtime_provider_results(
    output_dir: &Path,
) -> Result<Option<PreparedRuntimeProviderResults>, String> {
    let targets = nsdb::provider_runtime_result_targets(output_dir, None)?;
    if targets.is_empty() {
        return Ok(None);
    }
    let [target] = targets.as_slice() else {
        return Err("runtime provider dispatch currently requires exactly one target".to_owned());
    };
    let source_yir_path = find_source_yir(output_dir, &target.source_yir_fnv1a64)?;
    Ok(Some(PreparedRuntimeProviderResults {
        stream_path: nsdb::provider_runtime_result_stream_path(output_dir),
        source_yir_path,
        target_count: targets.len(),
        output_dir: output_dir.to_owned(),
    }))
}

fn find_source_yir(output_dir: &Path, expected_hash: &str) -> Result<PathBuf, String> {
    let mut matches = fs::read_dir(output_dir)
        .map_err(|error| format!("failed to enumerate runtime YIR artifacts: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("yir"))
        .filter(|path| {
            fs::read(path)
                .ok()
                .is_some_and(|source| fnv1a64_hex(&source) == expected_hash)
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches
        .into_iter()
        .next()
        .ok_or_else(|| format!("runtime provider source YIR `{expected_hash}` is unavailable"))
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}
