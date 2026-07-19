use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_source_owned_pointer_selection_to_branch_effect() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let left: ref Node = alloc_node(41, null());
            let right: ref Node = alloc_node(73, null());
            let selected: ref Node = select_owned_ptr(false, move(left), move(right));
            let value: i64 = load_value(selected);
            free(selected);
            return value;
          }
        }
        "#,
    )
    .unwrap();

    crate::nir_verify::verify_nir_module(&module).expect("owned pointer select should verify");
    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("owned pointer select should lower");
    let branch = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "branch_effect")
        .expect("owned pointer branch effect");
    assert_eq!(branch.op.args.get(1).map(String::as_str), Some("owned_ptr"));
    assert_eq!(
        branch
            .op
            .args
            .iter()
            .filter(|arg| arg.as_str() == "take_ptr_drop_other")
            .count(),
        2
    );

    let llvm = yir_lower_llvm::emit_module(&yir).expect("owned pointer select LLVM lowering");
    assert!(llvm.contains("phi ptr"));
    assert!(!llvm.contains("deferred lowering for cpu.branch_effect"));
}

#[test]
fn rejects_owned_pointer_selection_without_explicit_moves() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let left: ref Node = alloc_node(41, null());
            let right: ref Node = alloc_node(73, null());
            let selected: ref Node = select_owned_ptr(true, left, right);
            free(selected);
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("requires move(...) for both owned candidates"),
        "unexpected diagnostic: {error}"
    );
}

#[test]
fn rejects_aliasing_owned_pointer_selection() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let owner: ref Node = alloc_node(41, null());
            let selected: ref Node = select_owned_ptr(true, move(owner), move(owner));
            free(selected);
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let error = crate::nir_verify::verify_nir_module(&module).unwrap_err();
    assert!(
        error.contains("distinct owners"),
        "unexpected diagnostic: {error}"
    );
}

#[test]
fn rejects_reusing_either_consumed_candidate() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let left: ref Node = alloc_node(41, null());
            let right: ref Node = alloc_node(73, null());
            let selected: ref Node = select_owned_ptr(true, move(left), move(right));
            let invalid: i64 = load_value(right);
            free(selected);
            return invalid;
          }
        }
        "#,
    )
    .unwrap();

    let error = crate::nir_verify::verify_nir_module(&module).unwrap_err();
    assert!(
        error.contains("moved value"),
        "unexpected diagnostic: {error}"
    );
}
