use super::*;

pub(super) fn invariant_bindings<'a>(
    body: &[NirStmt],
    bindings: &'a BTreeMap<String, String>,
) -> BTreeSet<&'a str> {
    let mut invariant = bindings.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let mut pending = vec![body];
    while let Some(body) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                    invariant.remove(name.as_str());
                }
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    pending.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                NirStmt::While { body, .. } => pending.push(body),
                _ => {}
            }
        }
    }
    invariant
}

pub(super) fn ready(arg: &NirExpr, invariant: &BTreeSet<&str>) -> bool {
    if matches!(arg, NirExpr::Var(_)) {
        return true;
    }
    let mut root = arg;
    while let NirExpr::FieldAccess { base, .. } = root {
        root = base;
    }
    // These reads are materialized before the loop. Never snapshot a carried
    // record's field, or evaluate a computed argument on a zero-trip path.
    matches!(root, NirExpr::Var(name) if invariant.contains(name.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(root: NirExpr) -> NirExpr {
        NirExpr::FieldAccess {
            base: Box::new(root),
            field: "value".into(),
        }
    }

    #[test]
    fn scoped_arguments_only_hoist_ready_invariant_field_paths() {
        let invariant = BTreeSet::from(["input"]);
        assert!(ready(&field(NirExpr::Var("input".into())), &invariant));
        assert!(ready(
            &field(field(NirExpr::Var("input".into()))),
            &invariant
        ));
        assert!(ready(&NirExpr::Var("carry".into()), &invariant));
        for root in [
            NirExpr::Var("carry".into()),
            NirExpr::Var("missing".into()),
            NirExpr::Call {
                callee: "computed".into(),
                args: vec![],
            },
            NirExpr::Int(0),
        ] {
            assert!(!ready(&field(root), &invariant));
        }
        assert!(!ready(
            &NirExpr::CastBoolToI64(Box::new(NirExpr::Var("input".into()))),
            &invariant
        ));
    }

    #[test]
    fn scoped_arguments_exclude_nested_writes_and_unknown_roots() {
        let module = crate::frontend::parse_nuis_module(
            "mod cpu Main {
            fn test(input: i64, carry: i64) -> i64 {
                let i = 0;
                while i < 2 { if i > 0 { let carry = input; } let i = i + 1; }
                return carry;
            }
        }",
        )
        .unwrap();
        let bindings = ["input", "carry", "i"]
            .into_iter()
            .map(|name| (name.to_owned(), format!("node_{name}")))
            .collect();
        assert_eq!(
            invariant_bindings(&module.functions[0].body, &bindings),
            BTreeSet::from(["input"])
        );
    }
}
