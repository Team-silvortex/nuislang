use crate::{
    provider_prepared_input::PreparedProviderInput, provider_request::ProviderRequest,
    provider_sample_artifact::fnv1a64_hex, provider_sample_execute::resolve_provider_payload_path,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

pub(crate) struct PreparedProviderProcessAdapter {
    source_path: PathBuf,
    executable_path: PathBuf,
    pub(crate) contract: &'static str,
    pub(crate) executable_hash: String,
}

impl PreparedProviderProcessAdapter {
    pub(crate) fn executable_path(&self) -> &Path {
        &self.executable_path
    }
}

pub(crate) fn compile_objc_process_adapter(
    stem: &str,
    source: &str,
    contract: &'static str,
    frameworks: &[&str],
) -> Result<PreparedProviderProcessAdapter, String> {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let stem = format!("nuis-nsdb-{stem}-{}-{nonce}", std::process::id());
    let temp = std::env::temp_dir();
    let source_path = temp.join(format!("{stem}.m"));
    let executable_path = temp.join(stem);
    fs::write(&source_path, source)
        .map_err(|error| format!("failed to materialize provider adapter source: {error}"))?;
    let mut command = Command::new("clang");
    command.args(["-fobjc-arc", "-fblocks"]);
    for framework in frameworks {
        command.args(["-framework", framework]);
    }
    let compile = command
        .arg(&source_path)
        .arg("-o")
        .arg(&executable_path)
        .output()
        .map_err(|error| format!("failed to launch provider adapter compiler: {error}"))?;
    if !compile.status.success() {
        return Err(format!(
            "provider adapter compilation failed: {}",
            String::from_utf8_lossy(&compile.stderr).trim()
        ));
    }
    let executable = fs::read(&executable_path)
        .map_err(|error| format!("failed to hash provider adapter executable: {error}"))?;
    Ok(PreparedProviderProcessAdapter {
        source_path,
        executable_path,
        contract,
        executable_hash: fnv1a64_hex(&executable),
    })
}

impl Drop for PreparedProviderProcessAdapter {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.source_path);
        let _ = fs::remove_file(&self.executable_path);
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn worker_descriptor_argument(
    input: &PreparedProviderInput,
    descriptor_index: usize,
) -> Result<String, String> {
    match input.worker_adapter_argument().as_deref() {
        Some("path-fd") => Ok(format!("descriptor-path:{descriptor_index}")),
        Some(argument) => argument
            .strip_prefix("carrier-fd:")
            .map(|metadata| format!("descriptor-carrier:{descriptor_index}:{metadata}"))
            .ok_or_else(|| "provider worker adapter input argument is invalid".to_owned()),
        None => Err("provider worker adapter input has no descriptor argument".to_owned()),
    }
}

pub(crate) fn validate_provider_model_asset(
    output_dir: &Path,
    request: &ProviderRequest,
) -> Result<PathBuf, String> {
    let model = request
        .model_asset
        .as_ref()
        .ok_or_else(|| "CoreML provider request is missing a model asset descriptor".to_owned())?;
    let model_path = resolve_provider_payload_path(output_dir, &model.path)?;
    let model_bytes = fs::read(&model_path).map_err(|error| {
        format!(
            "failed to read provider model asset `{}`: {error}",
            model_path.display()
        )
    })?;
    if model_bytes.len() != model.byte_length || fnv1a64_hex(&model_bytes) != model.content_hash {
        return Err("provider model asset size/hash evidence mismatch".to_owned());
    }
    Ok(model_path)
}

pub(crate) fn provider_output_byte_length(request: &ProviderRequest) -> Option<usize> {
    request
        .output_comparison
        .as_ref()
        .map(|comparison| comparison.expected_byte_length)
        .or_else(|| {
            let element_bytes = match request.buffer.element_type.as_str() {
                "u8" => 1usize,
                "f32" => 4usize,
                _ => return None,
            };
            request
                .buffer
                .shape
                .iter()
                .try_fold(element_bytes, |bytes, dimension| {
                    bytes.checked_mul(*dimension)
                })
        })
}
