#[test]
fn lowers_provider_request_ingress_through_registered_data_nustar() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() {
            let request: i64 = provider_request_ingress(101, 501, 2, 20, 2020);
            print(request);
          }
        }
        "#,
    )
    .expect("provider request ingress source should compile");

    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|item| item == "official.data"));
    let ingress = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "data" && node.op.instruction == "provider_request_ingress")
        .expect("expected data.provider_request_ingress node");
    assert_eq!(ingress.op.args.len(), 5);
    assert!(artifacts
        .llvm_ir
        .contains("Nuis-owned provider request ingress"));
    assert!(!artifacts
        .llvm_ir
        .contains("deferred lowering for data.provider_request_ingress"));
}

#[test]
fn rejects_provider_request_ingress_with_incomplete_contract() {
    let error = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() {
            let request: i64 = provider_request_ingress(101, 501, 2, 20);
            print(request);
          }
        }
        "#,
    )
    .err()
    .expect("incomplete ingress contract should fail");

    assert!(error.contains("provider_request_ingress(...) expects 5 args"));
}

#[test]
fn materializes_exported_provider_worker_ingress_function() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          @export(name = "nuis_provider_worker_request_v1")
          fn worker_request(
            request: i64,
            descriptors: i64,
            descriptor_count: i64,
            provider: i64,
            capability: i64
          ) -> i64 {
            return provider_request_ingress(
              request,
              descriptors,
              descriptor_count,
              provider,
              capability
            );
          }

          fn main() -> i64 {
            return worker_request(101, 501, 2, 20, 2020);
          }
        }
        "#,
    )
    .expect("exported provider worker request should compile");

    assert!(artifacts.llvm_ir.contains(
        "define i64 @nuis_fn_worker_request(i64 %arg0, i64 %arg1, i64 %arg2, i64 %arg3, i64 %arg4)"
    ));
    assert!(artifacts
        .llvm_ir
        .contains("Nuis-owned provider request ingress"));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @nuis_fn_worker_request(i64 "));
    assert!(!artifacts.llvm_ir.contains("unsupported arity 5"));
}
