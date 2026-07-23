#[test]
fn lowers_nested_effectful_if_call_inside_selected_return_branch() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          extern "c" @host_symbol("provider_worker.reply")
          fn host_provider_worker_reply(status: i64) -> i64;

          fn close_if_requested(closing: i64) -> i64 {
            if closing != 0 {
              if host_provider_worker_reply(1) != 0 {
                return -4;
              }
              return 0;
            }
            return 7;
          }

          fn main() -> i64 {
            return close_if_requested(0);
          }
        }
        "#,
    )
    .expect("effectful return branch should compile");

    let guard = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "guard_host_call_return")
        .expect("expected branch-local host-call guard");
    assert!(guard.op.args.iter().any(|arg| arg == "compare_call_result"));
    assert!(guard
        .op
        .args
        .iter()
        .any(|arg| arg == "host_provider_worker_reply"));
    assert!(!artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "extern_call_i64"
            && node
                .op
                .args
                .iter()
                .any(|arg| arg == "host_provider_worker_reply")));

    let branch = artifacts
        .llvm_ir
        .find("guard_host_call_return_then.")
        .expect("guard branch");
    let guarded_ir = &artifacts.llvm_ir[branch..];
    let call = guarded_ir
        .find("call i64 @host_provider_worker_reply(i64")
        .expect("branch-local host call");
    let continuation = guarded_ir
        .find("\nguard_host_call_return_cont.")
        .expect("guard continuation");
    assert!(call < continuation);
}
