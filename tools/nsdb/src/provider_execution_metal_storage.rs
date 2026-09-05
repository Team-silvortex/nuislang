use crate::provider_request::ProviderRequest;
use yir_domain_shader::{ShaderFragmentStorage, ShaderFragmentStorageCapability};

pub(super) fn capability(
    request: &ProviderRequest,
) -> Result<Option<ShaderFragmentStorageCapability>, String> {
    let slot = super::draw::unique_scalar(request, "fragment_storage_slot")?;
    let count = super::draw::unique_scalar(request, "fragment_storage_count")?;
    let parse = |value: &crate::provider_request::ProviderScalarBinding| -> Result<usize, String> {
        let number = value
            .value
            .parse::<usize>()
            .map_err(|_| "invalid Metal storage capability integer")?;
        if value.value_type != "u64" || value.value != number.to_string() {
            return Err("invalid Metal storage capability integer type".to_owned());
        }
        Ok(number)
    };
    match (slot, count) {
        (None, None) => Ok(None),
        (Some(slot), Some(count)) => {
            let result = ShaderFragmentStorageCapability {
                slot: parse(slot)?,
                element_count: parse(count)?,
            };
            result.validate()?;
            Ok(Some(result))
        }
        _ => Err("incomplete Metal storage capability".to_owned()),
    }
}

pub(super) fn validate_upload(request: &ProviderRequest) -> Result<String, String> {
    let Some(capability) = capability(request)? else {
        if !request.runtime_uploads.is_empty() {
            return Err("Metal render has unadmitted runtime uploads".to_owned());
        }
        return Ok("none".to_owned());
    };
    let mut arguments = yir_domain_shader::ShaderDrawArguments {
        width: 1,
        height: 1,
        vertex_count: 3,
        instance_count: 1,
    }
    .to_dispatch();
    arguments.contract = yir_domain_shader::SHADER_FRAGMENT_STORAGE_CONTRACT.to_owned();
    arguments.uploads = request.runtime_uploads.clone();
    let storage = ShaderFragmentStorage::from_dispatch(&arguments)?;
    if storage.capability != capability {
        return Err("Metal runtime storage differs from admitted slot or array length".to_owned());
    }
    storage.upload.payload()?;
    Ok(format!(
        "{}:{}:{}",
        capability.slot, capability.element_count, storage.upload.content_hash
    ))
}
