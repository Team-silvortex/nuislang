pub(super) fn instrument(llvm: &str, loop_label: &str) -> String {
    let exit = format!("{}_exit", loop_label.strip_suffix("_body").unwrap());
    let mut result = String::new();
    let mut inside = false;
    let mut count = 0;
    let mut non_source_operands = std::collections::BTreeSet::new();
    let scoped = loop_label == "loop_while_i64_body";
    for line in llvm.lines() {
        if line.starts_with("define ") {
            non_source_operands.clear();
        }
        if let Some((register, _)) = line.trim().split_once(" = zext i1 ") {
            non_source_operands.insert(register);
        }
        if let Some((register, pointer)) = line.trim().split_once(" = load i64, ptr ") {
            if matches!(
                pointer.split(',').next(),
                Some("%nuis_helper_entries" | "%nuis_loop_work")
            ) {
                non_source_operands.insert(register);
            }
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
            && (!scoped || integer_source_comparison(line, &non_source_operands))
        {
            // Exclude ABI/neutral-guard bools and private work-counter loads.
            // Source comparisons still count, regardless of signedness or zero RHS.
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

fn integer_source_comparison(
    line: &str,
    non_source_operands: &std::collections::BTreeSet<&str>,
) -> bool {
    let Some((_, comparison)) = line.split_once(" = icmp ") else {
        return false;
    };
    let Some((_, operands)) = comparison.split_once(" i64 ") else {
        return false;
    };
    let Some((lhs, rhs)) = operands.split_once(", ") else {
        return false;
    };
    !non_source_operands.contains(lhs.trim()) && !non_source_operands.contains(rhs.trim())
}

#[test]
fn predicate_probe_excludes_work_guards_by_provenance_not_comparison_opcode() {
    let llvm = "define i64 @nuis_fn___nuis_scalar_iteration_0() {
  %entry = load i64, ptr %nuis_helper_entries, align 8
  %entry_ok = icmp uge i64 %entry, 1
  %loop = load i64, ptr %nuis_loop_work, align 8
  %loop_ok = icmp uge i64 %loop, %trips
  %bool64 = zext i1 %flag to i64
  %bool_ok = icmp ne i64 %bool64, 0
  %user = icmp uge i64 %value, 1
  %user_zero = icmp eq i64 %value, 0
  ret i64 0
}
";
    let probed = instrument(llvm, "loop_while_i64_body");
    assert_eq!(
        probed
            .matches(" = load volatile i64, ptr @probe_predicates")
            .count(),
        3
    );
    assert!(probed.contains("%user = icmp uge i64 %value, 1"));
    assert!(!probed.contains("%predicate_before_2"));
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
