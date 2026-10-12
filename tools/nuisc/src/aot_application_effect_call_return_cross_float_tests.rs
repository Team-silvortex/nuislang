use super::*;

const SOURCE: &str =
    include_str!("../tests/native_application_bridge/return_cross_float_private_pair.ns");

#[test]
fn native_return_cross_float_private_pair_keeps_exact_owned_layout_guarded_order_and_policy_vetoes()
{
    for (ty, child_ty) in [("f32", "f64"), ("f64", "f32")] {
        let (_, mut module, mode) = compiled(
            &SOURCE
                .replace("f32", "OWNER")
                .replace("f64", child_ty)
                .replace("OWNER", ty),
        );
        let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
        assert!(bridge.state_layout.source().ends_with(";result:bool}"));
        assert!(module
            .functions
            .iter()
            .any(|f| f.name.starts_with("__nuis_effect_call_")
                && f.result.as_ref().is_some_and(|result| result.ty == ty)));
        assert!(module.nodes.iter().any(|n| {
            n.op.instruction == "call_owned_struct"
                && n.op.args.get(1).is_some_and(|layout| {
                    layout.ends_with(&format!("{{owner:{ty};child:{child_ty}}}"))
                })
        }));
        assert!(!module.nodes.iter().any(|n| matches!(
            n.op.instruction.as_str(),
            "cast_f32_to_i64" | "cast_f64_to_i64" | "cast_i64_to_f32" | "cast_i64_to_f64"
        )));
        assert!(nonparticipating_coercions(&[
            "%1 = fpext float %arg0 to double",
            "%2 = fptosi double %1 to i64",
            "ret float %arg0"
        ]));
        for observed in ["ret double %1", "ret i64 %2"] {
            assert!(!nonparticipating_coercions(&[
                "%1 = fpext float %arg0 to double",
                "%2 = fptosi double %1 to i64",
                observed
            ]));
        }
        assert!(!nonparticipating_coercions(&[
            "%1 = fptrunc double %arg0 to float"
        ]));
        let mut definition = "";
        let mut body = vec![];
        for line in bridge.llvm_ir.lines() {
            if line.starts_with("define ") {
                definition = line;
                body.clear();
            } else if line == "}" && !definition.is_empty() {
                assert!(
                    nonparticipating_coercions(&body),
                    "live float conversion in {definition}"
                );
                definition = "";
            } else if !definition.is_empty() {
                body.push(line);
            }
        }
        for function in module
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_effect_call_"))
        {
            let sensitive = function
                .body_nodes
                .iter()
                .filter_map(|name| {
                    let node = module.nodes.iter().find(|n| &n.name == name).unwrap();
                    (node.op.instruction.starts_with("call_")
                        || matches!(node.op.instruction.as_str(), "guard_return" | "div" | "rem"))
                    .then_some(node)
                })
                .collect::<Vec<_>>();
            assert_eq!(sensitive[0].op.instruction, "guard_return");
            for pair in sensitive.windows(2) {
                let mut seen = std::collections::BTreeSet::new();
                let mut pending = vec![pair[0].name.as_str()];
                while let Some(from) = pending.pop() {
                    for edge in module.edges.iter().filter(|edge| {
                        edge.from == from
                            && matches!(edge.kind, yir_core::EdgeKind::Effect)
                            && function.body_nodes.contains(&edge.to)
                    }) {
                        if seen.insert(edge.to.as_str()) {
                            pending.push(edge.to.as_str());
                        }
                    }
                }
                assert!(
                    seen.contains(pair[1].name.as_str()),
                    "{} lacks an Effect path",
                    function.name
                );
            }
        }
        assert!(!bridge
            .llvm_ir
            .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        assert!(!bridge
            .llvm_ir
            .contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
        assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
        let empty = literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap();
        assert!(emit_for_packaging_mode(&module, &empty).is_err());
        module
            .edges
            .retain(|e| !matches!(e.kind, yir_core::EdgeKind::Effect));
        let error = emit_for_packaging_mode(&module, &mode).unwrap_err();
        assert!(error.contains("lacks dependency order"), "{error}");
    }
}

fn nonparticipating_coercions(lines: &[&str]) -> bool {
    let mentions = |line: &str, word: &str| {
        line.split(|c: char| !c.is_ascii_alphanumeric() && !"%_.$".contains(c))
            .any(|token| token == word)
    };
    for (index, line) in lines.iter().enumerate() {
        if line.contains("fptrunc ") {
            return false;
        }
        if !line.contains("fpext ") {
            continue;
        }
        let Some((word, _)) = line.trim().split_once(" = ") else {
            return false;
        };
        let uses: Vec<_> = lines
            .iter()
            .enumerate()
            .filter(|(i, l)| *i != index && mentions(l, word))
            .collect();
        // Legacy last-value bookkeeping may widen only into a dead integer result.
        let [(consumer, conversion)] = uses.as_slice() else {
            return false;
        };
        if !conversion.contains(" = fptosi double ") {
            return false;
        }
        let Some((unused, _)) = conversion.trim().split_once(" = ") else {
            return false;
        };
        if lines
            .iter()
            .enumerate()
            .any(|(i, l)| i != *consumer && mentions(l, unused))
        {
            return false;
        }
    }
    true
}
