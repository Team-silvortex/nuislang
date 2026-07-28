use crate::{
    provider_graph_output::{
        PROVIDER_COMPLETION_EVIDENCE_CONTRACT, PROVIDER_GLM_RELEASE_EVIDENCE_CONTRACT,
    },
    provider_sample_payload::{fnv1a64_hex, push_toml_string, PixelMagicNativeOutputSummary},
};
use std::collections::{BTreeMap, BTreeSet};

const COLLECTION_CONTRACT: &str = "nuis-provider-result-projection-collection-v1";
const PROJECTION_CONTRACT: &str = "nuis-provider-result-projection-v1";

struct ProviderResultProjection {
    source_function: String,
    source_node: String,
    producer_request_id: String,
    producer_output_buffer: String,
    expected_i64: i64,
    expected_content_hash: String,
}

struct ProviderResultObservation<'a> {
    request_id: &'a str,
    output_buffer: &'a str,
    element_type: &'a str,
    shape: &'a str,
    byte_length: &'a str,
    output_hash: &'a str,
    comparison_status: &'a str,
    completion_contract: &'a str,
    completion_token: &'a str,
    completion_status: &'a str,
    glm_release_contract: &'a str,
    glm_release_token: &'a str,
    glm_release_status: &'a str,
}

pub(crate) fn validate_and_render_result_projections(
    input_evidence: &str,
    native_outputs: &[PixelMagicNativeOutputSummary],
) -> Result<String, String> {
    let Some(projections) = parse_result_projections(input_evidence)? else {
        return Ok(String::new());
    };
    let observations = native_outputs
        .iter()
        .map(|output| ProviderResultObservation {
            request_id: &output.request_id,
            output_buffer: &output.output_binding_buffers,
            element_type: &output.output_binding_element_types,
            shape: &output.output_binding_shapes,
            byte_length: &output.output_binding_byte_lengths,
            output_hash: &output.hash,
            comparison_status: &output.comparison_status,
            completion_contract: &output.completion_evidence_contract,
            completion_token: &output.completion_token,
            completion_status: &output.completion_status,
            glm_release_contract: &output.glm_release_contract,
            glm_release_token: &output.glm_release_token,
            glm_release_status: &output.glm_release_status,
        })
        .collect::<Vec<_>>();
    validate_and_render(&projections, &observations)
}

fn parse_result_projections(
    evidence: &str,
) -> Result<Option<Vec<ProviderResultProjection>>, String> {
    let fields = evidence_fields(evidence);
    let Some(contract) = fields.get("provider_result_projection_collection_contract") else {
        return Ok(None);
    };
    if contract != COLLECTION_CONTRACT {
        return Err("provider result projection collection contract is invalid".to_owned());
    }
    let count = parse_usize(&fields, "provider_result_projection_count")?;
    if count == 0 || count > 64 {
        return Err("provider result projection count is invalid".to_owned());
    }
    let mut source_nodes = BTreeSet::new();
    let mut projections = Vec::with_capacity(count);
    for index in 0..count {
        let prefix = format!("provider_result_projection_{index}_");
        if required(&fields, &prefix, "contract")? != PROJECTION_CONTRACT
            || required(&fields, &prefix, "value_type")? != "i64"
            || parse_usize(&fields, &format!("{prefix}byte_offset"))? != 0
            || parse_usize(&fields, &format!("{prefix}byte_length"))? != 8
            || required(&fields, &prefix, "completion_requirement")?
                != PROVIDER_COMPLETION_EVIDENCE_CONTRACT
            || required(&fields, &prefix, "glm_release_requirement")?
                != PROVIDER_GLM_RELEASE_EVIDENCE_CONTRACT
        {
            return Err(format!(
                "provider result projection {index} contract or ABI is invalid"
            ));
        }
        let projection = ProviderResultProjection {
            source_function: required(&fields, &prefix, "source_function")?.to_owned(),
            source_node: required(&fields, &prefix, "source_node")?.to_owned(),
            producer_request_id: required(&fields, &prefix, "producer_request_id")?.to_owned(),
            producer_output_buffer: required(&fields, &prefix, "producer_output_buffer")?
                .to_owned(),
            expected_i64: required(&fields, &prefix, "expected_i64")?
                .parse()
                .map_err(|_| format!("provider result projection {index} i64 is invalid"))?,
            expected_content_hash: required(&fields, &prefix, "expected_content_hash")?.to_owned(),
        };
        if projection.source_function.is_empty()
            || projection.source_node.is_empty()
            || !source_nodes.insert(projection.source_node.clone())
            || projection.expected_content_hash
                != fnv1a64_hex(&projection.expected_i64.to_le_bytes())
        {
            return Err(format!(
                "provider result projection {index} identity or hash is invalid"
            ));
        }
        projections.push(projection);
    }
    Ok(Some(projections))
}

fn validate_and_render(
    projections: &[ProviderResultProjection],
    observations: &[ProviderResultObservation<'_>],
) -> Result<String, String> {
    let mut out = String::new();
    let mut canonical = Vec::with_capacity(projections.len());
    for (index, projection) in projections.iter().enumerate() {
        let observation = observations
            .iter()
            .find(|output| output.request_id == projection.producer_request_id)
            .ok_or_else(|| {
                format!(
                    "provider result projection `{}` has no completed producer",
                    projection.source_node
                )
            })?;
        if observation.output_buffer != projection.producer_output_buffer
            || observation.element_type != "i64"
            || observation.shape != "1x1"
            || observation.byte_length != "8"
            || observation.output_hash != projection.expected_content_hash
            || observation.comparison_status != "comparison-passed"
            || observation.completion_contract != PROVIDER_COMPLETION_EVIDENCE_CONTRACT
            || observation.completion_status != "worker-output-verified"
            || !observation
                .completion_token
                .starts_with("provider-completion:0x")
            || observation.glm_release_contract != PROVIDER_GLM_RELEASE_EVIDENCE_CONTRACT
            || observation.glm_release_status != "released-at-graph-close"
            || !observation.glm_release_token.starts_with("glm-release:0x")
        {
            return Err(format!(
                "provider result projection `{}` does not match completed output",
                projection.source_node
            ));
        }
        let prefix = format!("provider_result_projection_{index}_");
        for (name, value) in [
            ("contract", PROJECTION_CONTRACT),
            ("source_function", projection.source_function.as_str()),
            ("source_node", projection.source_node.as_str()),
            ("value_type", "i64"),
            (
                "producer_request_id",
                projection.producer_request_id.as_str(),
            ),
            (
                "producer_output_buffer",
                projection.producer_output_buffer.as_str(),
            ),
            ("value_i64", &projection.expected_i64.to_string()),
            ("output_hash", projection.expected_content_hash.as_str()),
            ("completion_token", observation.completion_token),
            ("glm_release_token", observation.glm_release_token),
            ("status", "verified"),
        ] {
            push_toml_string(&mut out, &format!("{prefix}{name}"), value);
        }
        canonical.push(format!(
            "{}:{}:{}:{}:{}:{}",
            projection.source_function,
            projection.source_node,
            projection.producer_request_id,
            projection.producer_output_buffer,
            observation.completion_token,
            observation.glm_release_token
        ));
    }
    let collection_hash = fnv1a64_hex(canonical.join(";").as_bytes());
    let mut header = String::new();
    push_toml_string(
        &mut header,
        "provider_result_projection_collection_contract",
        COLLECTION_CONTRACT,
    );
    push_toml_string(
        &mut header,
        "provider_result_projection_count",
        &projections.len().to_string(),
    );
    push_toml_string(
        &mut header,
        "provider_result_projection_collection_hash",
        &collection_hash,
    );
    push_toml_string(&mut header, "provider_result_projection_status", "verified");
    header.push_str(&out);
    Ok(header)
}

fn evidence_fields(input: &str) -> BTreeMap<String, String> {
    input
        .split(';')
        .filter_map(|field| field.split_once('='))
        .filter(|(key, value)| !key.is_empty() && !value.is_empty())
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn required<'a>(
    fields: &'a BTreeMap<String, String>,
    prefix: &str,
    name: &str,
) -> Result<&'a str, String> {
    fields
        .get(&format!("{prefix}{name}"))
        .map(String::as_str)
        .ok_or_else(|| format!("provider result projection is missing `{prefix}{name}`"))
}

fn parse_usize(fields: &BTreeMap<String, String>, key: &str) -> Result<usize, String> {
    fields
        .get(key)
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| format!("provider result projection has invalid `{key}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVIDENCE: &str = "provider_result_projection_collection_contract=nuis-provider-result-projection-collection-v1;\
provider_result_projection_count=1;\
provider_result_projection_0_contract=nuis-provider-result-projection-v1;\
provider_result_projection_0_source_function=main;\
provider_result_projection_0_source_node=selected;\
provider_result_projection_0_value_type=i64;\
provider_result_projection_0_producer_request_id=kernel.reduce;\
provider_result_projection_0_producer_output_buffer=output.reduce;\
provider_result_projection_0_byte_offset=0;\
provider_result_projection_0_byte_length=8;\
provider_result_projection_0_expected_i64=50;\
provider_result_projection_0_expected_content_hash=0xf71115b38f042bf7;\
provider_result_projection_0_completion_requirement=nuis-provider-completion-evidence-v1;\
provider_result_projection_0_glm_release_requirement=nuis-provider-glm-release-evidence-v1";

    fn observation<'a>(hash: &'a str) -> ProviderResultObservation<'a> {
        ProviderResultObservation {
            request_id: "kernel.reduce",
            output_buffer: "output.reduce",
            element_type: "i64",
            shape: "1x1",
            byte_length: "8",
            output_hash: hash,
            comparison_status: "comparison-passed",
            completion_contract: PROVIDER_COMPLETION_EVIDENCE_CONTRACT,
            completion_token: "provider-completion:0x1234",
            completion_status: "worker-output-verified",
            glm_release_contract: PROVIDER_GLM_RELEASE_EVIDENCE_CONTRACT,
            glm_release_token: "glm-release:0x5678",
            glm_release_status: "released-at-graph-close",
        }
    }

    #[test]
    fn verified_projection_binds_completed_output_and_lifecycle_tokens() {
        let projections = parse_result_projections(EVIDENCE)
            .unwrap()
            .expect("declared result projection");
        let rendered =
            validate_and_render(&projections, &[observation("0xf71115b38f042bf7")]).unwrap();
        assert!(rendered.contains("provider_result_projection_status = \"verified\""));
        assert!(rendered.contains("provider_result_projection_0_value_i64 = \"50\""));
        assert!(rendered.contains(
            "provider_result_projection_0_completion_token = \"provider-completion:0x1234\""
        ));
        assert!(rendered
            .contains("provider_result_projection_0_glm_release_token = \"glm-release:0x5678\""));
    }

    #[test]
    fn projection_rejects_output_or_declared_hash_drift() {
        let projections = parse_result_projections(EVIDENCE)
            .unwrap()
            .expect("declared result projection");
        assert!(validate_and_render(&projections, &[observation("0xdead")]).is_err());
        assert!(
            parse_result_projections(&EVIDENCE.replace("0xf71115b38f042bf7", "0xdead")).is_err()
        );
    }
}
