use crate::{ExecutionState, Node, Value, YirResultFamily};

pub const PROVIDER_COMPLETION_RECEIPT_CONTRACT: &str = "nuis-yir-provider-completion-receipt-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderCompletionReceipt {
    pub token: i64,
    pub completion_clock: i64,
    pub root: i64,
}

pub fn issue_provider_completion_receipt(
    family: YirResultFamily,
    resource: &str,
    source: &str,
    state: &str,
    completion_clock: i64,
) -> ProviderCompletionReceipt {
    let root = provider_completion_receipt_root(family, resource, source, state);
    ProviderCompletionReceipt {
        token: provider_completion_receipt_token(root, completion_clock),
        completion_clock,
        root,
    }
}

pub fn issue_observe_completion_receipt(
    node: &Node,
    state: &ExecutionState,
    family: YirResultFamily,
) -> Result<Option<ProviderCompletionReceipt>, String> {
    let Some(clock_name) = node.op.args.get(2) else {
        return Ok(None);
    };
    let completion_clock = state.expect_int(clock_name)?;
    Ok(Some(issue_provider_completion_receipt(
        family,
        &node.resource,
        &node.op.args[0],
        &node.op.args[1],
        completion_clock,
    )))
}

pub fn provider_completion_receipt_root(
    family: YirResultFamily,
    resource: &str,
    source: &str,
    state: &str,
) -> i64 {
    let canonical =
        format!("{PROVIDER_COMPLETION_RECEIPT_CONTRACT}\n{family}\n{resource}\n{source}\n{state}");
    positive_i64(fnv1a64(canonical.as_bytes()))
}

pub fn provider_completion_receipt_token(root: i64, completion_clock: i64) -> i64 {
    positive_i64((root as u64) ^ (completion_clock as u64)) | 1
}

pub fn project_provider_completion_receipt(
    receipt: Option<&ProviderCompletionReceipt>,
    field: &str,
) -> Result<Value, String> {
    let receipt = receipt.ok_or_else(|| {
        format!("result has no `{PROVIDER_COMPLETION_RECEIPT_CONTRACT}` metadata")
    })?;
    let value = match field {
        "completion_token" => receipt.token,
        "completion_clock" => receipt.completion_clock,
        "completion_root" => receipt.root,
        other => {
            return Err(format!(
                "unknown provider completion receipt field `{other}`"
            ))
        }
    };
    Ok(Value::Int(value))
}

fn positive_i64(value: u64) -> i64 {
    ((value & i64::MAX as u64).max(1)) as i64
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_receipts_are_stable_nonzero_and_clock_bound() {
        let first = issue_provider_completion_receipt(
            YirResultFamily::Shader,
            "shader0",
            "frame",
            "frame_ready",
            7,
        );
        let repeated = issue_provider_completion_receipt(
            YirResultFamily::Shader,
            "shader0",
            "frame",
            "frame_ready",
            7,
        );
        let next = issue_provider_completion_receipt(
            YirResultFamily::Shader,
            "shader0",
            "frame",
            "frame_ready",
            8,
        );

        assert_eq!(first, repeated);
        assert!(first.root > 0);
        assert!(first.token > 0);
        assert_ne!(first.token, next.token);
        assert_eq!(first.root, next.root);
        assert_eq!(next.completion_clock, 8);
    }
}
