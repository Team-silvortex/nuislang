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
