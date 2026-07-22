use crate::provider_sample_artifact::fnv1a64_hex;

pub(crate) const PROVIDER_WORKER_REQUEST_CONTRACT: &str =
    "nuis-provider-worker-request-envelope-v1";
pub(crate) const PROVIDER_WORKER_REQUEST_MAGIC: &str = "NUISPWU2";
pub(crate) const MAX_PROVIDER_WORKER_PAYLOAD_BYTES: usize = 60 * 1024;
pub(crate) const MAX_PROVIDER_WORKER_DESCRIPTORS: usize = 16;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProviderWorkerRequestEnvelope {
    pub(crate) lease_id: String,
    pub(crate) sequence: usize,
    pub(crate) request_id: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) payload_hash: String,
    pub(crate) descriptor_roles: Vec<String>,
}

pub(crate) fn encode_provider_worker_request(
    lease_id: &str,
    sequence: usize,
    request_id: &str,
    payload: &[u8],
    descriptor_roles: &[&str],
) -> Result<Vec<u8>, String> {
    validate_frame_token(lease_id, "lease id")?;
    validate_frame_token(request_id, "request id")?;
    validate_payload(payload)?;
    validate_descriptor_roles(descriptor_roles)?;
    let payload_hash = fnv1a64_hex(payload);
    let role_manifest = render_role_manifest(descriptor_roles);
    let header = format!(
        "{PROVIDER_WORKER_REQUEST_MAGIC}\t{lease_id}\t{sequence}\t{request_id}\t{}\t{payload_hash}\t{}\t{role_manifest}\n",
        payload.len(),
        descriptor_roles.len()
    );
    let mut frame = Vec::with_capacity(header.len() + payload.len());
    frame.extend_from_slice(header.as_bytes());
    frame.extend_from_slice(payload);
    Ok(frame)
}

pub(crate) fn decode_provider_worker_request(
    frame: &[u8],
    received_descriptor_count: usize,
) -> Result<ProviderWorkerRequestEnvelope, String> {
    let header_end = frame
        .iter()
        .position(|byte| *byte == b'\n')
        .ok_or_else(|| "provider worker request header is unterminated".to_owned())?;
    let header = std::str::from_utf8(&frame[..header_end])
        .map_err(|_| "provider worker request header is not UTF-8".to_owned())?;
    let fields = header.split('\t').collect::<Vec<_>>();
    if fields.len() != 8 || fields[0] != PROVIDER_WORKER_REQUEST_MAGIC {
        return Err("provider worker request envelope is invalid".to_owned());
    }
    validate_frame_token(fields[1], "lease id")?;
    validate_frame_token(fields[3], "request id")?;
    let sequence = parse_usize(fields[2], "sequence")?;
    let payload_length = parse_usize(fields[4], "payload length")?;
    let declared_descriptor_count = parse_usize(fields[6], "descriptor count")?;
    if declared_descriptor_count != received_descriptor_count {
        return Err("provider worker descriptor count mismatch".to_owned());
    }
    let descriptor_roles = parse_role_manifest(fields[7], declared_descriptor_count)?;
    let payload = &frame[header_end + 1..];
    if payload.len() != payload_length {
        return Err("provider worker payload length mismatch".to_owned());
    }
    validate_payload(payload)?;
    let payload_hash = fnv1a64_hex(payload);
    if fields[5] != payload_hash {
        return Err("provider worker payload hash mismatch".to_owned());
    }
    Ok(ProviderWorkerRequestEnvelope {
        lease_id: fields[1].to_owned(),
        sequence,
        request_id: fields[3].to_owned(),
        payload: payload.to_vec(),
        payload_hash,
        descriptor_roles,
    })
}

pub(crate) fn render_role_manifest(descriptor_roles: &[&str]) -> String {
    if descriptor_roles.is_empty() {
        "-".to_owned()
    } else {
        descriptor_roles.join(",")
    }
}

fn parse_role_manifest(value: &str, expected_count: usize) -> Result<Vec<String>, String> {
    let roles = if value == "-" {
        Vec::new()
    } else {
        value.split(',').map(str::to_owned).collect::<Vec<_>>()
    };
    if roles.len() != expected_count {
        return Err("provider worker descriptor role count mismatch".to_owned());
    }
    let borrowed = roles.iter().map(String::as_str).collect::<Vec<_>>();
    validate_descriptor_roles(&borrowed)?;
    Ok(roles)
}

fn validate_descriptor_roles(descriptor_roles: &[&str]) -> Result<(), String> {
    if descriptor_roles.len() > MAX_PROVIDER_WORKER_DESCRIPTORS {
        return Err("provider worker request has too many descriptor roles".to_owned());
    }
    for role in descriptor_roles {
        if role.is_empty()
            || !role.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-')
            })
        {
            return Err(format!(
                "provider worker descriptor role `{role}` is invalid"
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_frame_token(value: &str, name: &str) -> Result<(), String> {
    if value.is_empty() || value.contains(['\t', '\r', '\n']) {
        return Err(format!("provider worker {name} is not frame-safe"));
    }
    Ok(())
}

fn validate_payload(payload: &[u8]) -> Result<(), String> {
    if payload.len() > MAX_PROVIDER_WORKER_PAYLOAD_BYTES {
        return Err("provider worker request payload is too large".to_owned());
    }
    Ok(())
}

fn parse_usize(value: &str, name: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|error| format!("provider worker {name} is invalid: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_round_trips_opaque_payload_and_ordered_roles() {
        let payload = [0, b'\n', b'\t', 0xff, 17];
        let encoded = encode_provider_worker_request(
            "lease:test",
            3,
            "request.test",
            &payload,
            &["control.model", "input.primary", "output.result"],
        )
        .expect("encode");
        let decoded = decode_provider_worker_request(&encoded, 3).expect("decode");
        assert_eq!(decoded.payload, payload);
        assert_eq!(decoded.payload_hash, fnv1a64_hex(&payload));
        assert_eq!(
            decoded.descriptor_roles,
            ["control.model", "input.primary", "output.result"]
        );
    }

    #[test]
    fn envelope_rejects_payload_or_role_count_tampering() {
        let mut encoded = encode_provider_worker_request(
            "lease:test",
            0,
            "request.test",
            &[1, 2, 3],
            &["input.primary"],
        )
        .expect("encode");
        *encoded.last_mut().expect("payload") ^= 1;
        assert!(decode_provider_worker_request(&encoded, 1)
            .expect_err("hash mismatch")
            .contains("payload hash mismatch"));
        assert!(decode_provider_worker_request(&encoded, 0)
            .expect_err("role count mismatch")
            .contains("descriptor count mismatch"));
    }
}
