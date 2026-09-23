use super::*;

fn source(slots: usize) -> String {
    let base = typed_helper_values::source(slots);
    let start = base.find("            if p0 { return Packet").unwrap();
    let end = base[start..].find("\n        }").unwrap() + start;
    let body = &base[start..end];
    let initial = body
        .split("return ")
        .nth(1)
        .unwrap()
        .split("; }")
        .next()
        .unwrap();
    let advanced = body.rsplit("return ").next().unwrap().trim_end_matches(';');
    let mut advanced = advanced.to_owned();
    // Replace longest parameter names first (p1 must not rewrite p10).
    for index in (0..slots).rev() {
        advanced = advanced.replace(&format!("p{index}"), &format!("value.payload.f{index}"));
    }
    base.replace(
        body,
        &format!(
            "let value = {initial};
             if p0 {{ let value = relay(value); }}
             else {{ let value = {advanced}; }}
             if !p0 {{ let value = relay(value); }}
             return value;"
        ),
    )
}

#[test]
fn typed_local_choices_and_one_sided_rebindings_preserve_nested_snapshots_and_bits() {
    for slots in [1, 6, 63, 64] {
        let source = source(slots);
        typed_helper_values::check_values(slots, &source, 64);
    }
}

#[test]
fn typed_local_incompressible_capture_keeps_native_argument_bounds_explicit() {
    let fields = (0..64)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = (0..64)
        .map(|i| format!("f{i}: {i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "mod cpu Main {{
        struct State {{ {fields} }}
        @noinline fn relay(value: State) -> State {{ return value; }}
        fn start() -> State {{ return State {{ {values} }}; }}
        fn step(state: State) -> State {{
            let next = state;
            if state.f0 > 0 {{ let next = relay(state); }}
            return next;
        }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}"
    );
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let error = emit_registered(&compiled.yir, "counter").unwrap_err();
    // Sixty-four independent i64 values cannot share a word with the predicate.
    assert!(error.contains("function/argument bounds"), "{error}");
}
