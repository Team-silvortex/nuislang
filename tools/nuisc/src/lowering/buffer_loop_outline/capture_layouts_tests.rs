use super::*;

#[path = "../../../tests/native_application_bridge/scoped_record_fixture.rs"]
mod fixture;

#[test]
fn loop_branch_capture_authority_comes_from_generation_not_source_names() {
    let source = fixture::source(64, true)
        .replace("if i == 3 { break; }", "")
        .replace(
            "fn main()",
            "@noinline fn __nuis_buffer_branch_0(value: State, flag: bool) -> State { return value; } fn main()",
        );
    for reverse in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        if reverse {
            module.functions.reverse();
        }
        let source_parameters = module
            .functions
            .iter()
            .map(|f| (f.name.clone(), f.params.clone()))
            .collect::<BTreeMap<_, _>>();
        let outlined = outline_buffer_loops(&mut module).unwrap();
        let mut record_branches = 0;
        for function in &module.functions {
            if let Some(parameters) = source_parameters.get(&function.name) {
                assert_eq!(&function.params, parameters);
                assert!(!outlined.capture_plans.contains_key(&function.name));
            } else if outlined.guarded_functions.contains(&function.name)
                && function.params.iter().any(|p| p.ty.name == "State")
            {
                assert!(outlined.capture_plans.contains_key(&function.name));
                record_branches += 1;
            }
        }
        assert!(record_branches >= 2);
    }
}

#[test]
fn branch_capture_plans_keep_pure_layout_and_width_admission() {
    for count in [63, 64, 65] {
        let fields = (0..count)
            .map(|i| format!("f{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!("mod cpu Main {{ struct State {{ {fields} }} fn branch(value: State, flag: bool) -> State {{ return value; }} }}");
        let module = crate::frontend::parse_nuis_module(&source).unwrap();
        let layouts = control_values::TypedLayouts::collect(&module);
        let generated = BTreeSet::from(["branch".into()]);
        assert!(collect(&module, &BTreeSet::new(), &layouts).is_empty());
        assert_eq!(
            collect(&module, &generated, &layouts).contains_key("branch"),
            count == 64
        );
        if count != 64 {
            continue;
        }
        for rejected in [
            NirTypeRef {
                is_ref: true,
                ..scalar_type("State")
            },
            NirTypeRef {
                is_optional: true,
                ..scalar_type("State")
            },
            scalar_type("Buffer"),
            scalar_type("Unknown"),
        ] {
            let mut candidate = module.clone();
            candidate.functions[0].params[0].ty = rejected;
            assert!(collect(&candidate, &generated, &layouts).is_empty());
        }
    }
}
