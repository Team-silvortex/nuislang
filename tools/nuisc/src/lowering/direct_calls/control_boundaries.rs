use super::*;

pub(in crate::lowering) fn collect_guarded_loop_direct_call_functions(
    module: &NirModule,
) -> BTreeSet<String> {
    let pure_helpers = collect_pure_helper_functions(module);
    let function_names = module
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut reachable = BTreeSet::from(["main".to_owned()]);
    let mut frontier = vec!["main".to_owned()];
    while let Some(name) = frontier.pop() {
        let Some(function) = function_map.get(name.as_str()) else {
            continue;
        };
        for called in function_called_functions(function, &function.body, &function_names) {
            if reachable.insert(called.clone()) {
                frontier.push(called);
            }
        }
    }

    module
        .functions
        .iter()
        .filter(|function| function.name != "main")
        .filter(|function| !function.is_async)
        .filter(|function| reachable.contains(&function.name))
        .filter(|function| supports_direct_call_signature(function))
        .filter(|function| stmts_contain_guarded_loop_boundary(&function.body, &pure_helpers))
        .map(|function| function.name.clone())
        .collect()
}

fn stmts_contain_guarded_loop_boundary(stmts: &[NirStmt], pure_helpers: &BTreeSet<String>) -> bool {
    stmts.iter().any(|stmt| match stmt {
        NirStmt::While { body, .. } => {
            prepare_guarded_loop_body(body, pure_helpers).is_some()
                || stmts_contain_guarded_loop_boundary(body, pure_helpers)
        }
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            stmts_contain_guarded_loop_boundary(then_body, pure_helpers)
                || stmts_contain_guarded_loop_boundary(else_body, pure_helpers)
        }
        _ => false,
    })
}
