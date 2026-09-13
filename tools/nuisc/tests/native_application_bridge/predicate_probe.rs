pub(super) fn instrument(llvm: &str, loop_label: &str) -> String {
    let exit = format!("{}_exit", loop_label.strip_suffix("_body").unwrap());
    let mut result = String::new();
    let mut inside = false;
    let mut count = 0;
    for line in llvm.lines() {
        if line.starts_with(loop_label) && line.ends_with(':') {
            inside = true;
        }
        if line.starts_with(&exit) || line == "}" {
            inside = false;
        }
        if inside && line.contains(" = icmp ") {
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
    assert!(llvm.contains("carry_predicate_merge"));
    result.push_str("\n@probe_predicates = internal global i64 0, align 8\ndefine void @probe_predicate_evidence() {\n  call void @nuis_debug_print_i64(i64 83)\n  %count = load volatile i64, ptr @probe_predicates\n  call void @nuis_debug_print_i64(i64 %count)\n  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\n");
    result
}
