use super::*;

#[test]
fn typed_sparse_literal_returns_reduce_private_carries_without_widening_native_limits() {
    for width in [10, 63, 64] {
        let source = fixture::literal_return_source(width);
        let compiled = nuisc::pipeline::compile_source(&source).unwrap();
        assert_eq!(carried_widths(&compiled.yir), [width - 4, width]);
        let computed = source
            .replace(
                "mod cpu Main {",
                "mod cpu Main { @noinline fn opaque(value: i64) -> i64 { return value; }",
            )
            .replace("extra9: 99", "extra9: opaque(99)");
        let compiled = nuisc::pipeline::compile_source(&computed).unwrap();
        assert_eq!(carried_widths(&compiled.yir), [width - 3, width + 1]);
        if width == 64 {
            let project = Project::with_source(&computed);
            let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
            let error = emit_registered(&compiled.yir, "counter").unwrap_err();
            assert!(
                error.contains("65 carried words exceed the 64-word native limit"),
                "{error}"
            );
        }
    }
}

#[test]
fn typed_sparse_literal_returns_execute_all_words_with_exact_visible_exits() {
    for width in [10, 63, 64] {
        let source = fixture::literal_return_source(width);
        let cases = [0, 1, 2, 3, 4]
            .into_iter()
            .map(|limit| {
                let initial = initial(width, limit);
                let mut event = event(&initial, limit);
                event[9] = 99;
                if limit > 0 {
                    event[2] = 17;
                    event[8] = limit + 1;
                }
                (
                    vec![initial[0], initial[1], initial[2], limit],
                    vec![initial, event.clone(), event],
                )
            })
            .collect();
        if width == 10 {
            typed_record_inputs::check_flattened(&source, cases);
        } else {
            typed_record_inputs::check_compact(&source, cases, width - 7);
        }
    }
}

#[test]
fn typed_sparse_literal_returns_execute_mixed_constant_storage_without_heap_aggregation() {
    let source = fixture::literal_return_source(64).replace(
        "right: state.right",
        "right: Leaf { value: 3.5, gain: 4.5, tag: i32_from_i64(9), enabled: false }",
    );
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert_eq!(carried_widths(&compiled.yir), [60, 60]);
    let cases = [0, 1, 2, 3, 4]
        .into_iter()
        .map(|limit| {
            let initial = initial(64, limit);
            let mut event = event(&initial, limit);
            event[9] = 99;
            event[4..8].copy_from_slice(&[3.5_f64.to_bits(), u64::from(4.5_f32.to_bits()), 9, 0]);
            if limit > 0 {
                event[2] = 17;
                event[8] = limit + 1;
            }
            (
                vec![initial[0], initial[1], initial[2], limit],
                vec![initial, event.clone(), event],
            )
        })
        .collect();
    typed_record_inputs::check_compact(&source, cases, 57);
}
