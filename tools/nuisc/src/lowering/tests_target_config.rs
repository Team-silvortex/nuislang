use std::path::Path;

use super::{lower_nir_to_yir, lower_nir_to_yir_builtin_cpu_with_target, LoweringTargetConfig};
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_cpu_target_config_and_resource_kind_for_selected_abi() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return 7;
          }
        }
        "#,
    )
    .unwrap();
    let target = LoweringTargetConfig {
        abi: "cpu.x86_64.sysv64".to_owned(),
        machine_arch: "x86_64".to_owned(),
        machine_os: "linux".to_owned(),
        object_format: "elf".to_owned(),
        calling_abi: "sysv64".to_owned(),
        clang_target: "x86_64-unknown-linux-gnu".to_owned(),
    };

    let yir = lower_nir_to_yir_builtin_cpu_with_target(&module, Some(&target)).unwrap();

    assert_eq!(yir.resources[0].kind.raw, "cpu.x86_64");
    let target_config = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "target_config")
        .unwrap();
    assert_eq!(
        target_config.op.args,
        vec![
            "x86_64".to_owned(),
            "cpu.x86_64.sysv64".to_owned(),
            "128".to_owned()
        ]
    );
    let contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "lowering_cpu_target_contract_type")
        .unwrap();
    assert_eq!(
        contract.op.args,
        vec!["arch=symbol:x86_64;abi=symbol:cpu.x86_64.sysv64;vector_bits=i64:128".to_owned()]
    );
    assert!(yir.edges.iter().any(|edge| {
        edge.from == "lowering_cpu_target_contract_type" && edge.to == "lowering_cpu_target_config"
    }));
    assert_eq!(
        yir.node_lanes
            .get("lowering_cpu_target_contract_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("lowering_cpu_target_config")
            .map(String::as_str),
        Some("contract")
    );
}

#[test]
fn rejects_unregistered_lowering_abi_target() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return 1;
          }
        }
        "#,
    )
    .unwrap();
    let manifest =
        crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), "cpu").unwrap();
    let target = LoweringTargetConfig {
        abi: "cpu.missing.abi".to_owned(),
        machine_arch: "x86_64".to_owned(),
        machine_os: "linux".to_owned(),
        object_format: "elf".to_owned(),
        calling_abi: "sysv64".to_owned(),
        clang_target: "x86_64-unknown-linux-gnu".to_owned(),
    };

    let error = lower_nir_to_yir(&module, &manifest, Some(&target)).unwrap_err();

    assert!(error.contains("does not declare required ABI `cpu.missing.abi`"));
}

#[test]
fn rejects_nurs_extern_when_lowering_target_is_plain_c_abi() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "nurs" fn host_color_bias(value: i64) -> i64;

          fn main() -> i64 {
            return host_color_bias(7);
          }
        }
        "#,
    )
    .unwrap();
    let target = LoweringTargetConfig {
        abi: "cpu.x86_64.sysv64".to_owned(),
        machine_arch: "x86_64".to_owned(),
        machine_os: "linux".to_owned(),
        object_format: "elf".to_owned(),
        calling_abi: "sysv64".to_owned(),
        clang_target: "x86_64-unknown-linux-gnu".to_owned(),
    };

    let error = lower_nir_to_yir_builtin_cpu_with_target(&module, Some(&target)).unwrap_err();

    assert!(
        error.contains("extern ABI `nurs` is not supported by lowering target `cpu.x86_64.sysv64`")
    );
}

#[test]
fn allows_nurs_extern_when_lowering_target_declares_nurs_bridge() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          extern "nurs" fn host_color_bias(value: i64) -> i64;

          fn main() -> i64 {
            return host_color_bias(7);
          }
        }
        "#,
    )
    .unwrap();
    let target = LoweringTargetConfig {
        abi: "cpu.arm64.nurs.c-abi.v1".to_owned(),
        machine_arch: "arm64".to_owned(),
        machine_os: "darwin".to_owned(),
        object_format: "mach-o".to_owned(),
        calling_abi: "aapcs64-darwin".to_owned(),
        clang_target: "aarch64-apple-darwin".to_owned(),
    };

    let yir = lower_nir_to_yir_builtin_cpu_with_target(&module, Some(&target)).unwrap();

    assert!(yir.nodes.iter().any(|node| {
        node.op.module == "cpu"
            && node.op.instruction == "extern_call_i64"
            && node.op.args.first().is_some_and(|abi| abi == "nurs")
    }));
}
