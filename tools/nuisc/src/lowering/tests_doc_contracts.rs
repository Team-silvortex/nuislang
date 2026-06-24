use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_doc_comments_into_yir_contract_nodes() {
    let module = parse_nuis_module(
        r#"
/// module contract docs
mod cpu Main {
    /// main contract docs
    fn main() -> i64 {
        42
    }
}
"#,
    )
    .expect("module parses and lowers to NIR");

    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("NIR lowers to YIR");
    let module_doc = yir
        .nodes
        .iter()
        .find(|node| node.name == "doc_contract_module_cpu_main")
        .expect("module doc contract node exists");
    assert_eq!(module_doc.op.module, "cpu");
    assert_eq!(module_doc.op.instruction, "text");
    assert!(module_doc.op.args[0].contains("schema=nuis-yir-doc-contract-v1"));
    assert!(module_doc.op.args[0].contains("scope=module"));
    assert!(module_doc.op.args[0].contains("path=cpu.Main"));
    assert!(module_doc.op.args[0].contains("docs=module contract docs"));
    assert_eq!(
        yir.node_lanes
            .get("doc_contract_module_cpu_main")
            .map(String::as_str),
        Some("contract")
    );

    let function_doc = yir
        .nodes
        .iter()
        .find(|node| node.name == "doc_contract_function_cpu_main__main")
        .expect("function doc contract node exists");
    assert!(function_doc.op.args[0].contains("scope=function"));
    assert!(function_doc.op.args[0].contains("path=cpu.Main::main"));
    assert!(function_doc.op.args[0].contains("docs=main contract docs"));
    assert!(function_doc.op.args[0].contains("signature=fn main() -> i64"));
    assert_eq!(
        yir.node_lanes
            .get("doc_contract_function_cpu_main__main")
            .map(String::as_str),
        Some("contract")
    );
}
