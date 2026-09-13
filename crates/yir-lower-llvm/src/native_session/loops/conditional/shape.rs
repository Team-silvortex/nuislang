use super::{branch_source, condition_source, Node, MAX_SCALAR_SLOTS};

pub(super) const MAX_CONDITION_NODES: usize = 127;
pub(super) const MAX_CONDITION_DEPTH: usize = 32;
const MAX_ARGS: usize = 5 + MAX_SCALAR_SLOTS * (3 + 2 * MAX_CONDITION_NODES);

pub(super) fn validate(node: &Node) -> Result<(), String> {
    let fail = |message: &str| format!("native conditional loop `{}` {message}", node.name);
    let bound = || fail("exceeds its conditional-carry shape/bound");
    if node.op.args.len() < 9 || node.op.args.len() > MAX_ARGS {
        return Err(bound());
    }
    let take = |cursor: &mut usize| {
        let value = node
            .op
            .args
            .get(*cursor)
            .ok_or_else(|| fail("has truncated conditional metadata"))?;
        *cursor += 1;
        Ok::<_, String>(value.as_str())
    };
    let mut cursor = 5;
    let mut carries = 0;
    while cursor < node.op.args.len() {
        if carries == MAX_SCALAR_SLOTS {
            return Err(bound());
        }
        take(&mut cursor)?;
        let root_always = node
            .op
            .args
            .get(cursor)
            .is_some_and(|kind| kind == "always");
        // Check depth and node count iteratively before invoking the shared
        // recursive parser. RHS value names are opaque, even if named and/keep.
        let mut pending = vec![1];
        let mut nodes = 0;
        while let Some(depth) = pending.pop() {
            nodes += 1;
            if depth > MAX_CONDITION_DEPTH || nodes > MAX_CONDITION_NODES {
                return Err(bound());
            }
            let kind = take(&mut cursor)?;
            if matches!(kind, "and" | "or") {
                pending.extend([depth + 1, depth + 1]);
            } else {
                if !condition_source(kind, carries, MAX_SCALAR_SLOTS) {
                    return Err(fail("has noncanonical conditional metadata"));
                }
                if kind != "always" {
                    take(&mut cursor)?;
                }
            }
        }
        if root_always {
            let candidate = node
                .op
                .args
                .get(cursor)
                .ok_or_else(|| fail("has missing carry branches"))?;
            // Only the existing induction-seed placeholder may be ignored;
            // a real branch kind with that same name remains a branch kind.
            if !branch_source(candidate, carries, MAX_SCALAR_SLOTS)
                && Some(candidate) == node.op.args.first()
            {
                cursor += 1;
            }
        }
        for _ in 0..2 {
            if !branch_source(take(&mut cursor)?, carries, MAX_SCALAR_SLOTS) {
                return Err(fail("has noncanonical conditional metadata"));
            }
        }
        carries += 1;
    }
    Ok(())
}
