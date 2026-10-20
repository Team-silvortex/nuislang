use crate::backend_artifacts::{
    render_kernel_graph_artifact_stub, render_kernel_graph_summary,
    render_kernel_stage_artifact_stub, render_kernel_stage_summary, render_shader_artifact_stub,
    render_shader_stage_summary, shader_backend_checks,
};
use crate::host_compile::c_shim_source;
use crate::host_ffi::{HostFfiArgType, HostFfiReturnType, HostFfiSignature};
use crate::host_ffi_stub::render_host_ffi_stubs;
use crate::host_frames::RuntimeFrameSupport;
use crate::host_objc::{objc_host_source, ObjcHostSourceSpec};
use crate::host_plan::{
    extract_fabric_boot_plan, extract_primary_fabric_binding, render_fabric_boot_plan,
};
use crate::host_source::bytes_to_c_array;
use crate::shader_codegen::{compile_backend_expr, map_wgsl_expr_to_backend, ShaderTargetFlavor};
use std::collections::BTreeMap;
use std::path::PathBuf;
use yir_lower_contract::analyze_kernel_lowering;
use yir_lower_contract::analyze_shader_lowering;

fn fingerprint(parts: impl IntoIterator<Item = String>) -> String {
    let mut source = String::new();
    for part in parts {
        source.push_str(&format!("{}:", part.len()));
        source.push_str(&part);
    }
    yir_core::provider_runtime_ipc::hash_bytes(source.as_bytes())
}

fn shader_stage() -> yir_lower_contract::ShaderStageContract {
    let module = yir_syntax::parse_module(include_str!(
        "../../../examples/yir/shader/shader_overlay_demo.yir"
    ))
    .unwrap();
    analyze_shader_lowering(&module).stages.remove(0)
}

fn ir_stage(stage: &str, expr: &str) -> yir_lower_contract::NustarContractStage {
    yir_lower_contract::NustarContractStage {
        stage: stage.to_owned(),
        function: format!("frame_{stage}"),
        node_kind: "function".to_owned(),
        execution_domain: "shader".to_owned(),
        time_mode: "sync".to_owned(),
        contract_family: "shader".to_owned(),
        time_domain: "frame".to_owned(),
        glm_scope: "stage".to_owned(),
        instructions: vec![],
        terminator: yir_lower_contract::NustarContractTerminator {
            op: "return".to_owned(),
            expr: expr.to_owned(),
        },
    }
}

#[test]
fn shader_artifact_sources_preserve_pre_split_fingerprints() {
    let base = shader_stage();
    let mut wgsl = base.clone();
    wgsl.wgsl_entry = Some("frame_fs".to_owned());
    wgsl.wgsl_source = Some(
        "@fragment fn frame_fs() -> @location(0) vec4<f32> {\nlet tint: vec4<f32> = vec4<f32>(0.2, 0.4, 0.6, 1.0);\nreturn tint;\n}"
            .to_owned(),
    );
    let mut ir = wgsl.clone();
    ir.shader_ir_stages
        .push(ir_stage("fragment", "vec4<f32>(0.2, 0.4, 0.6, 1.0)"));
    let expected = [
        "0x3b84ba1236d974a3",
        "0x21b1eb63c2aa3ac3",
        "0xcab31718cf3924ee",
    ];
    for (stage, expected) in [base, wgsl, ir].into_iter().zip(expected) {
        let summary = render_shader_stage_summary(&stage);
        let variants = stage.backend_variants();
        assert!(variants.len() >= 5);
        let parts = variants.iter().map(|v| {
            let body = render_shader_artifact_stub(&stage, v, &summary);
            format!(
                "{}\n{}\n{:?}",
                v.backend,
                body,
                shader_backend_checks(&stage, v.backend, &body)
            )
        });
        assert_eq!(fingerprint(parts), expected);
    }
}

#[test]
fn kernel_artifact_sources_preserve_pre_split_fingerprints() {
    for (source, expected) in [
        include_str!("../../../examples/yir/kernel/kernel_tensor_demo.yir"),
        include_str!("../../../examples/yir/kernel/kernel_typed_scalars.yir"),
    ]
    .into_iter()
    .zip(["0xade143275bec3a40", "0x231e765bfe3c0ff2"])
    {
        let module = yir_syntax::parse_module(source).unwrap();
        let contract = analyze_kernel_lowering(&module);
        assert!(contract.has_kernel_work());
        let mut parts = vec![contract.render_text(), contract.render_package_manifest()];
        for stage in &contract.stages {
            let summary = render_kernel_stage_summary(stage);
            for v in stage.backend_variants() {
                parts.push(render_kernel_stage_artifact_stub(stage, &v, &summary));
            }
        }
        for graph in &contract.graphs {
            let summary = render_kernel_graph_summary(graph);
            for v in graph.backend_variants() {
                parts.push(render_kernel_graph_artifact_stub(graph, &v, &summary));
            }
        }
        assert_eq!(fingerprint(parts), expected);
    }
}

#[test]
fn host_sources_and_fabric_plan_preserve_pre_split_fingerprints() {
    let mut symbols = BTreeMap::new();
    symbols.insert(
        "host_color_bias".to_owned(),
        HostFfiSignature {
            abi: "c".to_owned(),
            return_type: HostFfiReturnType::I64,
            arg_types: vec![HostFfiArgType::I64],
        },
    );
    let stubs = render_host_ffi_stubs(&symbols);
    let mut parts = vec![
        c_shim_source(&symbols),
        bytes_to_c_array(b"P6\n1 1\n255\n\0\x02\xff"),
    ];
    let module = yir_syntax::parse_module(include_str!(
        "../../../examples/yir/data/data_fabric_demo.yir"
    ))
    .unwrap();
    let contract = analyze_shader_lowering(&module);
    let binding = extract_primary_fabric_binding(&contract);
    let plan = extract_fabric_boot_plan(&module, binding.as_ref(), Some("worker"));
    let rendered = render_fabric_boot_plan(&plan);
    parts.push(rendered.clone());
    let runtime = RuntimeFrameSupport {
        staticlib_path: PathBuf::from("runtime_host.a"),
        embedded_module_bytes: bytes_to_c_array(b"yir 0.1\n"),
        frame_scale: 3,
    };
    for runtime_frame_support in [None, Some(&runtime)] {
        for fabric_worker_core in [None, Some(2)] {
            parts.push(objc_host_source(ObjcHostSourceSpec {
                window_title: "preview \"quoted\"",
                window_width: 640,
                window_height: 480,
                fabric_worker_core,
                fabric_table_id: Some("table"),
                fabric_host_resource: Some("host"),
                fabric_render_resource: Some("render"),
                fabric_boot_plan: &rendered,
                fabric_boot_plan_len: plan.len(),
                embedded_fallback_ppm_bytes: Some("0x50, 0x36, 0x00,"),
                runtime_frame_support,
                host_ffi_stubs: &stubs,
            }));
        }
    }
    assert_eq!(fingerprint(parts), "0x1ddf31f55891a6a1");
}

#[test]
fn shader_expression_sources_preserve_pre_split_fingerprint() {
    let mut parts = vec![];
    for flavor in [
        ShaderTargetFlavor::OpenGl,
        ShaderTargetFlavor::VulkanGlsl,
        ShaderTargetFlavor::Metal,
        ShaderTargetFlavor::Hlsl,
    ] {
        for expr in [
            "vec4<f32>(0.2, 0.4, 0.6, 1.0)",
            "mix(vec4<f32>(uv, 0.0, 1.0), vec4<f32>(1.0), fract(uv.x))",
            "textureSample(tex, samp, uv) + textureSample(tex, samp, uv)",
            "clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))",
        ] {
            for uses_texture in [false, true] {
                parts.push(format!(
                    "{:?}",
                    compile_backend_expr(expr, flavor, uses_texture)
                ));
                parts.push(format!(
                    "{:?}",
                    map_wgsl_expr_to_backend(expr, flavor, uses_texture)
                ));
            }
        }
    }
    assert_eq!(fingerprint(parts), "0x1f2c6fe9ab836ca9");
}
