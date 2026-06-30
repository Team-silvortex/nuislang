use super::*;
use crate::lowering::loop_carries::tail_recursive_prev_carry_binding;
use crate::lowering::loop_purity::{
    collect_inlineable_pure_helper_exprs, extract_pure_branch_binding, substitute_branch_binding,
};
use nuis_semantics::model::NirParam;

#[path = "tail_recursion_canonical.rs"]
mod tail_recursion_canonical;
#[path = "tail_recursion_extract.rs"]
mod tail_recursion_extract;
#[path = "tail_recursion_rewrite.rs"]
mod tail_recursion_rewrite;
#[cfg(test)]
#[path = "tail_recursion_tests.rs"]
mod tail_recursion_tests;

use tail_recursion_extract::extract_self_tail_recursive_shape;
use tail_recursion_rewrite::{
    is_self_tail_recursive_loop_shape, rewrite_self_tail_recursive_loop_body,
};

pub(super) fn rewrite_self_tail_recursive_functions(module: &NirModule) -> NirModule {
    let pure_helpers = collect_pure_helper_functions(module);
    let inlineable_pure_helpers = collect_inlineable_pure_helper_exprs(module);
    let pure_helper_blocks = collect_pure_helper_blocks(module);
    let mut rewritten = module.clone();
    for function in &mut rewritten.functions {
        if let Some(body) = rewrite_self_tail_recursive_function(
            function,
            &pure_helpers,
            &inlineable_pure_helpers,
            &pure_helper_blocks,
        ) {
            function.body = body;
        }
    }
    rewritten
}

fn rewrite_self_tail_recursive_function(
    function: &NirFunction,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<Vec<NirStmt>> {
    if function.params.is_empty() {
        return None;
    }

    let (recurse_condition, base_return, recursive_step) =
        extract_self_tail_recursive_shape(function, pure_helpers)?;
    let loop_body = rewrite_self_tail_recursive_loop_body(function, recursive_step)?;

    if !is_self_tail_recursive_loop_shape(
        &recurse_condition,
        &loop_body,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    ) {
        return None;
    }

    Some(vec![
        NirStmt::While {
            condition: recurse_condition,
            body: loop_body,
        },
        NirStmt::Return(Some(base_return)),
    ])
}

pub(super) enum SelfTailRecursiveStep {
    Linear(Vec<NirExpr>),
    FlowBreak {
        condition: NirExpr,
        recursive_step: Box<SelfTailRecursiveStep>,
    },
    PostFlowBreak {
        condition: NirExpr,
        recursive_step: Box<SelfTailRecursiveStep>,
        control_carry_index: usize,
    },
    Branch {
        condition: NirExpr,
        then_args: Vec<NirExpr>,
        else_args: Vec<NirExpr>,
    },
}

pub(super) enum SelfTailRecursiveDecisionTree {
    Leaf(Vec<NirExpr>),
    Branch {
        condition: NirExpr,
        then_tree: Box<SelfTailRecursiveDecisionTree>,
        else_tree: Box<SelfTailRecursiveDecisionTree>,
    },
}
