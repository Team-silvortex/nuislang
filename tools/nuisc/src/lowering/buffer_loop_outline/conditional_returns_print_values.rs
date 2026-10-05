use super::*;

pub(super) struct PrintValue<'a> {
    pub(super) value: &'a NirExpr,
    pub(super) params: Option<Vec<NirParam>>,
}

pub(super) fn prepare<'a>(
    value: &'a NirExpr,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<PrintValue<'a>> {
    let params = match value {
        NirExpr::Int(_) => None,
        NirExpr::Var(name) if scope.get(name) == Some(&scalar_type("i64")) => None,
        _ => {
            // Bound the complete original argument before recursive purity/type
            // inference. Ordinary leaves cannot hide new logical-root authority.
            if !conditional_values::prefix::expression(value)
                || control_values::value_type(value, scope, catalog, layouts)
                    != Some(scalar_type("i64"))
            {
                return None;
            }
            let mut inputs = BTreeSet::new();
            control_values::collect_inputs(value, &mut inputs);
            if inputs
                .iter()
                .any(|name| !scope.get(name).is_some_and(scalar))
            {
                return None;
            }
            Some(captured_params(inputs, scope))
        }
    };
    Some(PrintValue { value, params })
}

pub(super) fn install(
    print: PrintValue<'_>,
    condition: &NirExpr,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    output: &mut Vec<NirStmt>,
) -> NirExpr {
    let Some(captures) = print.params else {
        return print.value.clone();
    };
    let mut parameter_names = captures.iter().map(|param| param.name.clone()).collect();
    let gate = branches::fresh_name("__nuis_print_condition", &mut parameter_names);
    let params = std::iter::once(NirParam {
        name: gate.clone(),
        ty: scalar_type("bool"),
    })
    .chain(captures)
    .collect::<Vec<_>>();
    let mut args = vec![condition.clone()];
    args.extend(
        params[1..]
            .iter()
            .map(|param| NirExpr::Var(param.name.clone())),
    );
    let name = branches::fresh_name("__nuis_conditional_print_value", names);
    helpers.push(helper(
        name.clone(),
        params,
        vec![NirStmt::If {
            condition: NirExpr::Var(gate),
            then_body: vec![NirStmt::Return(Some(print.value.clone()))],
            else_body: vec![NirStmt::Return(Some(NirExpr::Int(0)))],
        }],
    ));
    // Only ready parent atoms cross the call boundary. All checked/call-backed
    // argument work stays in its selected helper arm; printing stays outside.
    let value = branches::fresh_name("__nuis_print_value", bindings);
    output.push(NirStmt::Let {
        name: value.clone(),
        ty: Some(scalar_type("i64")),
        value: NirExpr::Call { callee: name, args },
    });
    NirExpr::Var(value)
}
