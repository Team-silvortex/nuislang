use sha2::{Digest, Sha256};

use super::{
    CompilerComponentBuild, CompilerComponentDependency, COMPILER_COMPONENT_BUILD_PROTOCOL,
    COMPILER_COMPONENT_DEPENDENCY_CLOSURE_CONTRACT,
};

pub(super) fn dependency_closure_identity(
    component_id: &str,
    dependencies: &[CompilerComponentDependency],
) -> String {
    let mut hash = Sha256::new();
    hash_field(&mut hash, COMPILER_COMPONENT_BUILD_PROTOCOL.as_bytes());
    hash_field(
        &mut hash,
        COMPILER_COMPONENT_DEPENDENCY_CLOSURE_CONTRACT.as_bytes(),
    );
    hash_field(&mut hash, component_id.as_bytes());
    hash_field(&mut hash, &(dependencies.len() as u64).to_le_bytes());
    for dependency in dependencies {
        hash_field(&mut hash, &(dependency.ordinal as u64).to_le_bytes());
        hash_field(&mut hash, dependency.kind.as_bytes());
        hash_field(&mut hash, dependency.identity.as_bytes());
        hash_field(&mut hash, &(dependency.content_bytes as u64).to_le_bytes());
        hash_field(&mut hash, dependency.content_sha256.as_bytes());
    }
    finish_hash(hash)
}

pub(super) fn reproducible_build_identity(build: &CompilerComponentBuild) -> String {
    let mut hash = Sha256::new();
    for value in [
        build.protocol.as_bytes(),
        build.driver_contract.as_bytes(),
        build.stage_role.as_bytes(),
        build.bootstrap_subset_protocol.as_bytes(),
        build.component_id.as_bytes(),
        build.component_domain.as_bytes(),
        build.component_unit.as_bytes(),
        build.producer_id.as_bytes(),
        build.compiler_image_sha256.as_bytes(),
        build.stage_handoff_bundle_sha256.as_bytes(),
        build.native_binary_sha256.as_bytes(),
        build.dependency_closure_sha256.as_bytes(),
        build.reproducible_identity_contract.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        build.compiler_image_bytes,
        build.native_binary_bytes,
        build.dependency_count,
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    finish_hash(hash)
}

pub(super) fn component_build_identity(build: &CompilerComponentBuild) -> String {
    let mut hash = Sha256::new();
    for value in [
        build.protocol.as_bytes(),
        build.driver_contract.as_bytes(),
        build.stage_role.as_bytes(),
        build.bootstrap_subset_protocol.as_bytes(),
        build.component_id.as_bytes(),
        build.component_domain.as_bytes(),
        build.component_unit.as_bytes(),
        build.producer_id.as_bytes(),
        build.compiler_image_sha256.as_bytes(),
        build.stage_handoff_file.as_bytes(),
        build.stage_handoff_bundle_sha256.as_bytes(),
        build.build_manifest_file.as_bytes(),
        build.build_manifest_sha256.as_bytes(),
        build.compiled_artifact_file.as_bytes(),
        build.compiled_artifact_sha256.as_bytes(),
        build.native_binary_file.as_bytes(),
        build.native_binary_sha256.as_bytes(),
        build.dependency_closure_contract.as_bytes(),
        build.dependency_closure_sha256.as_bytes(),
        build.reproducible_identity_contract.as_bytes(),
        build.reproducible_build_sha256.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        build.compiler_image_bytes,
        build.build_manifest_bytes,
        build.compiled_artifact_bytes,
        build.native_binary_bytes,
        build.dependency_count,
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    finish_hash(hash)
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    finish_hash(Sha256::new_with_prefix(bytes))
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

fn finish_hash(hash: Sha256) -> String {
    let digest = hash.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
