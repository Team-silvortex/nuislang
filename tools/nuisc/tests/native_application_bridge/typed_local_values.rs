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
    for slots in [1, 6, 63] {
        let source = source(slots);
        typed_helper_values::check_values(slots, &source, 64);
    }
}

#[test]
fn typed_local_full_record_capture_keeps_native_argument_bounds_explicit() {
    let project = Project::with_source(&source(64));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let error = emit_registered(&compiled.yir, "counter").unwrap_err();
    // A full 64-leaf capture plus its separate predicate requires 65 arguments.
    assert!(error.contains("function/argument bounds"), "{error}");
}
