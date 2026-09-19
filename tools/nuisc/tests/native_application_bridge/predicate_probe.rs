pub(super) fn instrument(llvm: &str, loop_label: &str) -> String {
    let exit = format!("{}_exit", loop_label.strip_suffix("_body").unwrap());
    let mut result = String::new();
    let mut inside = false;
    let mut count = 0;
    let mut packed_bools = std::collections::BTreeSet::new();
    let scoped = loop_label == "loop_while_i64_body";
    for line in llvm.lines() {
        if line.starts_with("define ") {
            packed_bools.clear();
        }
        if let Some((register, _)) = line.trim().split_once(" = zext i1 ") {
            packed_bools.insert(register);
        }
        if scoped && line.starts_with("define ") {
            inside = [
                "@nuis_fn___nuis_buffer_branch_",
                "@nuis_fn___nuis_scalar_iteration_",
                "@nuis_fn___nuis_scalar_predicate_",
            ]
            .iter()
            .any(|name| line.contains(name));
        }
        if !scoped && line.starts_with(loop_label) && line.ends_with(':') {
            inside = true;
        }
        if (!scoped && line.starts_with(&exit)) || line == "}" {
            inside = false;
        }
        if inside
            && line.contains(" = icmp ")
            && (!scoped || integer_source_comparison(line, &packed_bools))
        {
            // Exclude comparisons of packed bools used by ABI normalization
            // and neutral guards. Integer equality/inequality with zero still counts.
            // Volatile observations make eager evaluation detectable even when
            // two predicate trees happen to return identical carry values.
            result.push_str(&format!("  %predicate_before_{count} = load volatile i64, ptr @probe_predicates\n  %predicate_after_{count} = add i64 %predicate_before_{count}, 1\n  store volatile i64 %predicate_after_{count}, ptr @probe_predicates\n"));
            count += 1;
        }
        if line.trim() == "call void @llvm.trap()" {
            result.push_str("  call void @probe_predicate_evidence()\n");
        }
        result.push_str(line);
        result.push('\n');
    }
    assert!(
        count >= 2,
        "compound predicate must emit multiple comparisons"
    );
    assert!(scoped || llvm.contains("carry_predicate_merge"));
    result.push_str("\n@probe_predicates = internal global i64 0, align 8\ndefine void @probe_predicate_evidence() {\n  call void @nuis_debug_print_i64(i64 83)\n  %count = load volatile i64, ptr @probe_predicates\n  call void @nuis_debug_print_i64(i64 %count)\n  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\n");
    result
}

fn integer_source_comparison(line: &str, packed_bools: &std::collections::BTreeSet<&str>) -> bool {
    let Some((_, comparison)) = line.split_once(" = icmp ") else {
        return false;
    };
    let Some((_, operands)) = comparison.split_once(" i64 ") else {
        return false;
    };
    let Some((lhs, rhs)) = operands.split_once(", ") else {
        return false;
    };
    !packed_bools.contains(lhs.trim()) && !packed_bools.contains(rhs.trim())
}

#[test]
fn predicate_probe_distinguishes_integer_tests_from_packed_boolean_guards() {
    let packed = std::collections::BTreeSet::from(["%flag64", "%false64"]);
    for op in ["eq", "ne", "slt", "sle", "sgt", "sge"] {
        assert!(integer_source_comparison(
            &format!("  %r = icmp {op} i64 %integer, 0"),
            &packed
        ));
        assert!(integer_source_comparison(
            &format!("  %r = icmp {op} i64 0, %integer"),
            &packed
        ));
        assert!(!integer_source_comparison(
            &format!("  %r = icmp {op} i64 %flag64, 0"),
            &packed
        ));
        assert!(!integer_source_comparison(
            &format!("  %r = icmp {op} i64 %integer, %false64"),
            &packed
        ));
    }
    assert!(!integer_source_comparison(
        "  %r = icmp eq i1 %flag, false",
        &packed
    ));
}
