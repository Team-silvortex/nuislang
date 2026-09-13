use super::{emit_loop_compare, fresh_block, fresh_reg, CpuLoopScalarKind};
use yir_domain_cpu::LoopCondExpr;

pub(crate) fn rhs_names(condition: &LoopCondExpr) -> Vec<&str> {
    let mut pending = vec![condition];
    let mut names = Vec::new();
    while let Some(condition) = pending.pop() {
        match condition {
            LoopCondExpr::Leaf { rhs: Some(rhs), .. } => names.push(rhs.as_str()),
            LoopCondExpr::Leaf { rhs: None, .. } => {}
            LoopCondExpr::Binary { lhs, rhs, .. } => {
                pending.push(rhs);
                pending.push(lhs);
            }
        }
    }
    names
}

pub(crate) fn resolve_rhs(
    condition: &LoopCondExpr,
    resolve: &mut impl FnMut(&str) -> Option<String>,
) -> Option<LoopCondExpr> {
    Some(match condition {
        LoopCondExpr::Leaf { kind, rhs } => LoopCondExpr::Leaf {
            kind: kind.clone(),
            rhs: match rhs {
                Some(rhs) => Some(resolve(rhs)?),
                None => None,
            },
        },
        LoopCondExpr::Binary { op, lhs, rhs } => LoopCondExpr::Binary {
            op: op.clone(),
            lhs: Box::new(resolve_rhs(lhs, resolve)?),
            rhs: Box::new(resolve_rhs(rhs, resolve)?),
        },
    })
}

pub(crate) struct State<'a> {
    pub kind: CpuLoopScalarKind,
    pub current: &'a str,
    pub previous_current: &'a str,
    pub carries: &'a [String],
    pub previous_carries: &'a [String],
}

pub(crate) fn emit(
    condition: &LoopCondExpr,
    state: &State<'_>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    next_block: &mut usize,
) -> Result<String, String> {
    if let LoopCondExpr::Leaf { kind, rhs } = condition {
        return emit_leaf(kind, rhs.as_deref(), state, body, next_reg);
    }
    let selected = fresh_block(next_block, "carry_predicate_true");
    let skipped = fresh_block(next_block, "carry_predicate_false");
    let merge = fresh_block(next_block, "carry_predicate_merge");
    emit_branch(
        condition,
        state,
        [&selected, &skipped],
        body,
        next_reg,
        next_block,
    )?;
    body.push(format!("{selected}:"));
    body.push(format!("  br label %{merge}"));
    body.push(format!("{skipped}:"));
    body.push(format!("  br label %{merge}"));
    body.push(format!("{merge}:"));
    let result = fresh_reg(next_reg);
    body.push(format!(
        "  {result} = phi i1 [ true, %{selected} ], [ false, %{skipped} ]"
    ));
    Ok(result)
}

fn emit_branch(
    condition: &LoopCondExpr,
    state: &State<'_>,
    targets: [&str; 2],
    body: &mut Vec<String>,
    next_reg: &mut usize,
    next_block: &mut usize,
) -> Result<(), String> {
    match condition {
        LoopCondExpr::Leaf { kind, rhs } => {
            let value = emit_leaf(kind, rhs.as_deref(), state, body, next_reg)?;
            body.push(format!(
                "  br i1 {value}, label %{}, label %{}",
                targets[0], targets[1]
            ));
        }
        LoopCondExpr::Binary { op, lhs, rhs } => {
            let is_and = match op.as_str() {
                "and" => true,
                "or" => false,
                _ => return Err(format!("unsupported carry condition operator `{op}`")),
            };
            let rhs_block = fresh_block(next_block, &format!("carry_predicate_{op}_rhs"));
            let left_targets = if is_and {
                [rhs_block.as_str(), targets[1]]
            } else {
                [targets[0], rhs_block.as_str()]
            };
            emit_branch(lhs, state, left_targets, body, next_reg, next_block)?;
            body.push(format!("{rhs_block}:"));
            emit_branch(rhs, state, targets, body, next_reg, next_block)?;
        }
    }
    Ok(())
}

fn emit_leaf(
    kind: &str,
    rhs: Option<&str>,
    state: &State<'_>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Result<String, String> {
    if kind == "always" && rhs.is_none() {
        return Ok("true".into());
    }
    let (source, compare) = kind
        .rsplit_once('_')
        .ok_or_else(|| format!("invalid carry condition `{kind}`"))?;
    let lookup = |prefix: &str, carries: &'_ [String]| {
        source
            .strip_prefix(prefix)?
            .parse::<usize>()
            .ok()
            .and_then(|i| carries.get(i))
            .cloned()
    };
    let lhs = match source {
        "current" => Some(state.current.to_owned()),
        "prev_current" => Some(state.previous_current.to_owned()),
        _ => {
            lookup("prev_carry", state.previous_carries).or_else(|| lookup("carry", state.carries))
        }
    }
    .ok_or_else(|| format!("unavailable carry condition source `{kind}`"))?;
    let rhs = rhs.ok_or_else(|| format!("missing carry condition rhs for `{kind}`"))?;
    emit_loop_compare(body, next_reg, state.kind, compare, &lhs, rhs)
}
