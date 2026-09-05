use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) struct PreparedRuntimeProviderResults {
    pub(crate) stream_path: PathBuf,
    pub(crate) source_yir_path: PathBuf,
    pub(crate) target_count: usize,
    pub(crate) invocation_count: usize,
    #[cfg(test)]
    pub(crate) report: nsdb::ProviderSampleExecuteReport,
}

impl PreparedRuntimeProviderResults {
    pub(crate) fn bind_to_command(&self, command: &mut Command) {
        command.env(
            yir_runtime_host::PROVIDER_RESULT_STREAM_ENV,
            &self.stream_path,
        );
    }

    pub(crate) fn print_text(&self) {
        println!("  runtime_provider_result_targets: {}", self.target_count);
        println!(
            "  runtime_provider_result_invocations: {}",
            self.invocation_count
        );
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
    if targets.len() != 1 {
        return Err(
            "bounded runtime provider result preparation currently requires exactly one target"
                .to_owned(),
        );
    }
    let source_hashes = targets
        .iter()
        .map(|target| target.source_yir_fnv1a64.as_str())
        .collect::<BTreeSet<_>>();
    if source_hashes.len() != 1 {
        return Err("runtime provider targets mix source YIR identities".to_owned());
    }
    let source_hash = source_hashes
        .first()
        .expect("non-empty runtime target source set");
    let (source_yir_path, source_yir) = find_source_yir(output_dir, source_hash)?;
    let invocation_counts = targets
        .iter()
        .map(|target| {
            yir_runtime_host::count_module_node_executions(
                &source_yir,
                &target.module,
                &target.instruction,
                &target.node,
                &target.resource,
            )
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if invocation_counts.len() != 1 {
        return Err("runtime provider targets require different invocation counts".to_owned());
    }
    let invocation_count = *invocation_counts
        .first()
        .expect("non-empty runtime invocation count set");
    let provider_families = targets
        .iter()
        .map(|target| target.provider_family.as_str())
        .collect::<BTreeSet<_>>();
    let provider_family_filter = (provider_families.len() == 1)
        .then(|| *provider_families.first().expect("single provider family"));
    let report = nsdb::execute_provider_samples_for_runtime(
        output_dir,
        provider_family_filter,
        invocation_count,
    )?;
    if report.output_payload_count == 0 {
        return Err("runtime provider execution produced no output payload".to_owned());
    }
    let stream_path = nsdb::provider_runtime_result_stream_path(output_dir);
    if !stream_path.is_file() {
        return Err("runtime provider execution did not persist its result stream".to_owned());
    }
    Ok(Some(PreparedRuntimeProviderResults {
        stream_path,
        source_yir_path,
        target_count: targets.len(),
        invocation_count,
        #[cfg(test)]
        report,
    }))
}

fn find_source_yir(output_dir: &Path, expected_hash: &str) -> Result<(PathBuf, String), String> {
    let mut matches = fs::read_dir(output_dir)
        .map_err(|error| format!("failed to enumerate runtime YIR artifacts: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("yir"))
        .filter_map(|path| {
            let source = fs::read_to_string(&path).ok()?;
            (fnv1a64_hex(source.as_bytes()) == expected_hash).then_some((path, source))
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| left.0.cmp(&right.0));
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
