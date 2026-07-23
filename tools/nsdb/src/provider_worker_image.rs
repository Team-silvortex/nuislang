use crate::provider_runner_registry::{
    select_provider_worker_image_registration, ProviderWorkerImageRegistration,
};
use std::{fs, path::PathBuf, process::Command};

pub(crate) const PROVIDER_WORKER_IMAGE_RESOLVER_CONTRACT: &str =
    "nuis-provider-worker-image-resolver-v1";

pub(crate) struct ResolvedProviderWorkerImage {
    pub(crate) resolver_contract: &'static str,
    pub(crate) registration: ProviderWorkerImageRegistration,
    pub(crate) binary_path: PathBuf,
    pub(crate) cache_key: String,
    pub(crate) cache_status: &'static str,
}

impl ResolvedProviderWorkerImage {
    pub(crate) fn command(&self) -> Command {
        let mut command = Command::new(&self.binary_path);
        command.env(
            "NUIS_PROVIDER_WORKER_PROVIDER_KEY",
            self.registration.provider_key.to_string(),
        );
        command.env(
            "NUIS_PROVIDER_WORKER_CAPABILITY_HASH",
            self.registration.capability_hash.to_string(),
        );
        command.env(
            "NUIS_PROVIDER_WORKER_DESCRIPTOR_CONTRACT",
            self.registration.descriptor_capability.contract,
        );
        command.env(
            "NUIS_PROVIDER_WORKER_MAX_SEMANTIC_DESCRIPTORS",
            self.registration
                .descriptor_capability
                .max_semantic_descriptors
                .to_string(),
        );
        command.env(
            "NUIS_PROVIDER_WORKER_MAX_CONTROL_DESCRIPTORS",
            self.registration
                .descriptor_capability
                .max_control_descriptors
                .to_string(),
        );
        command.env(
            "NUIS_PROVIDER_WORKER_OUTPUT_DESCRIPTOR_CONTRACT",
            self.registration.output_descriptor_capability.contract,
        );
        command.env(
            "NUIS_PROVIDER_WORKER_MAX_OUTPUT_DESCRIPTORS",
            self.registration
                .output_descriptor_capability
                .max_output_descriptors
                .to_string(),
        );
        command
    }
}

pub(crate) fn resolve_provider_worker_image(
    provider_family: &str,
    output_dir: &std::path::Path,
) -> Result<ResolvedProviderWorkerImage, String> {
    let registration = select_provider_worker_image_registration(provider_family)
        .ok_or_else(|| format!("provider family `{provider_family}` has no worker image"))?;
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = workspace_root.join(registration.source_path);
    if !source.is_file() {
        return Err(format!(
            "registered provider worker source `{}` is unavailable",
            registration.source_path
        ));
    }
    let cache_key = nuisc::cache::compute_compile_cache_key_with_identity(
        &source,
        None,
        registration.cache_identity,
    )?;
    if let Some(entry) = nuisc::cache::lookup_compile_cache(&cache_key)? {
        nuisc::cache::restore_compile_cache(&entry, output_dir)?;
        let binary_path = worker_binary_path(&source, output_dir)?;
        if binary_path.is_file() {
            return Ok(ResolvedProviderWorkerImage {
                resolver_contract: PROVIDER_WORKER_IMAGE_RESOLVER_CONTRACT,
                registration,
                binary_path,
                cache_key: cache_key.key,
                cache_status: "hit",
            });
        }
    }
    fs::create_dir_all(output_dir).map_err(|error| {
        format!(
            "failed to create provider worker output `{}`: {error}",
            output_dir.display()
        )
    })?;
    let pipeline = nuisc::pipeline::compile_source_path(&source)?;
    let target = nuisc::aot::host_cpu_build_target();
    let linked = nuisc::aot::write_and_link(
        &source,
        output_dir,
        &pipeline.ast,
        &pipeline.nir,
        &pipeline.yir,
        &pipeline.llvm_ir,
        &target,
    )?;
    nuisc::cache::store_compile_cache(&cache_key, output_dir)?;
    Ok(ResolvedProviderWorkerImage {
        resolver_contract: PROVIDER_WORKER_IMAGE_RESOLVER_CONTRACT,
        registration,
        binary_path: PathBuf::from(linked.binary_path),
        cache_key: cache_key.key,
        cache_status: "miss",
    })
}

fn worker_binary_path(
    source: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<PathBuf, String> {
    let file_name = source
        .file_stem()
        .ok_or_else(|| "provider worker source has no file stem".to_owned())?;
    Ok(output_dir.join(file_name))
}
