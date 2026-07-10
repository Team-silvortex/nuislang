use crate::ExecutionResourceBinding;

use super::super::{domain_resource_capability_label, ExecutionResourceKind, Executor};
use super::support::*;

#[test]
fn executor_default_kernel_plan_uses_buffer_and_dispatch_resources() {
    let adapter = PassiveAdapter;
    let unit = sample_kernel_unit();
    let payload = sample_kernel_payload();
    let host_plan = sample_host_plan("kernel", "official.kernel", "hetero-submit-bridge");
    let bridge_registry = sample_bridge_registry(
        "kernel",
        "official.kernel",
        "vulkan",
        "vulkan.discrete-or-integrated-gpu",
    );
    let prepared =
        prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);

    let plan = Executor.plan(&prepared).unwrap();

    assert_eq!(
        plan.phases[0].ir_sidecar_summary.as_deref(),
        Some(
            "capability_owner=kernel-nustar native_ir=spirv1.6 tensor_lowering=storage-buffer-tensor-view dispatch_lowering=compute-grid-or-indirect result_lowering=storage-buffer-result"
        )
    );
    assert_eq!(
        plan.phases[0].action.input_handles,
        vec!["kernel.buffer".to_owned(), "queue.slot".to_owned()]
    );
    assert_eq!(
        plan.phases[1].action.output_handles,
        vec!["dispatch.handle".to_owned()]
    );
    assert_eq!(
        plan.phases[1].action.resource_bindings,
        vec![
            ExecutionResourceBinding {
                key: "bridge_surface".to_owned(),
                kind: ExecutionResourceKind::Bridge,
                capability_label: Some(domain_resource_capability_label(
                    "kernel",
                    Some("vulkan.discrete-or-integrated-gpu"),
                    "bridge_surface",
                    &ExecutionResourceKind::Bridge,
                )),
                value: "host-ffi.bridge.kernel".to_owned()
            },
            ExecutionResourceBinding {
                key: "scheduler_binding".to_owned(),
                kind: ExecutionResourceKind::Scheduler,
                capability_label: Some(domain_resource_capability_label(
                    "kernel",
                    Some("vulkan.discrete-or-integrated-gpu"),
                    "scheduler_binding",
                    &ExecutionResourceKind::Scheduler,
                )),
                value: "hetero-submit-bridge".to_owned()
            },
            ExecutionResourceBinding {
                key: "backend_summary".to_owned(),
                kind: ExecutionResourceKind::Metadata,
                capability_label: Some(domain_resource_capability_label(
                    "kernel",
                    Some("vulkan.discrete-or-integrated-gpu"),
                    "backend_summary",
                    &ExecutionResourceKind::Metadata,
                )),
                value: "kernel_ir = \"spirv1.6\"".to_owned()
            },
            ExecutionResourceBinding {
                key: "lowering_capabilities".to_owned(),
                kind: ExecutionResourceKind::Metadata,
                capability_label: Some(domain_resource_capability_label(
                    "kernel",
                    Some("vulkan.discrete-or-integrated-gpu"),
                    "lowering_capabilities",
                    &ExecutionResourceKind::Metadata,
                )),
                value: "capability_owner=kernel-nustar native_ir=spirv1.6 tensor_lowering=storage-buffer-tensor-view dispatch_lowering=compute-grid-or-indirect result_lowering=storage-buffer-result".to_owned()
            },
            ExecutionResourceBinding {
                key: "kernel_buffer".to_owned(),
                kind: ExecutionResourceKind::Buffer,
                capability_label: Some(domain_resource_capability_label(
                    "kernel",
                    Some("vulkan.discrete-or-integrated-gpu"),
                    "kernel_buffer",
                    &ExecutionResourceKind::Buffer,
                )),
                value: "slot:kernel.buffer".to_owned()
            },
            ExecutionResourceBinding {
                key: "dispatch_handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    "kernel",
                    Some("vulkan.discrete-or-integrated-gpu"),
                    "dispatch_handle",
                    &ExecutionResourceKind::Handle,
                )),
                value: "slot:dispatch.handle".to_owned()
            },
            ExecutionResourceBinding {
                key: "result_buffer".to_owned(),
                kind: ExecutionResourceKind::Buffer,
                capability_label: Some(domain_resource_capability_label(
                    "kernel",
                    Some("vulkan.discrete-or-integrated-gpu"),
                    "result_buffer",
                    &ExecutionResourceKind::Buffer,
                )),
                value: "slot:result.buffer".to_owned()
            }
        ]
    );
}

#[test]
fn executor_default_shader_plan_uses_shader_and_frame_resources() {
    let adapter = PassiveAdapter;
    let unit = sample_shader_unit();
    let payload = sample_shader_payload();
    let host_plan = sample_host_plan("shader", "official.shader", "render-submit-bridge");
    let bridge_registry = sample_bridge_registry(
        "shader",
        "official.shader",
        "metal",
        "metal.apple-silicon-gpu",
    );
    let prepared =
        prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);

    let trace = Executor.execute_prepared(&prepared).unwrap();
    let plan = Executor.plan(&prepared).unwrap();

    assert_eq!(
        plan.phases[0].ir_sidecar_summary.as_deref(),
        Some(
            "capability_owner=shader-nustar native_ir=msl2.4 pipeline_lowering=metal-render-pipeline-state resource_lowering=argument-buffer-table texture_lowering=texture2d-sampler-argument"
        )
    );

    assert_eq!(
        trace.events[0].action.input_handles,
        vec!["shader.buffer".to_owned(), "frame.target".to_owned()]
    );
    assert_eq!(
        trace.events[1].action.output_handles,
        vec!["draw.handle".to_owned()]
    );
    assert_eq!(
        trace.events[3].action.resolved_resources,
        vec![
            ExecutionResourceBinding {
                key: "bridge_surface".to_owned(),
                kind: ExecutionResourceKind::Bridge,
                capability_label: Some(domain_resource_capability_label(
                    "shader",
                    Some("metal.apple-silicon-gpu"),
                    "bridge_surface",
                    &ExecutionResourceKind::Bridge,
                )),
                value: "host-ffi.bridge.shader".to_owned()
            },
            ExecutionResourceBinding {
                key: "scheduler_binding".to_owned(),
                kind: ExecutionResourceKind::Scheduler,
                capability_label: Some(domain_resource_capability_label(
                    "shader",
                    Some("metal.apple-silicon-gpu"),
                    "scheduler_binding",
                    &ExecutionResourceKind::Scheduler,
                )),
                value: "render-submit-bridge".to_owned()
            },
            ExecutionResourceBinding {
                key: "backend_summary".to_owned(),
                kind: ExecutionResourceKind::Metadata,
                capability_label: Some(domain_resource_capability_label(
                    "shader",
                    Some("metal.apple-silicon-gpu"),
                    "backend_summary",
                    &ExecutionResourceKind::Metadata,
                )),
                value: "shader_ir = \"msl2.4\"".to_owned()
            },
            ExecutionResourceBinding {
                key: "lowering_capabilities".to_owned(),
                kind: ExecutionResourceKind::Metadata,
                capability_label: Some(domain_resource_capability_label(
                    "shader",
                    Some("metal.apple-silicon-gpu"),
                    "lowering_capabilities",
                    &ExecutionResourceKind::Metadata,
                )),
                value: "capability_owner=shader-nustar native_ir=msl2.4 pipeline_lowering=metal-render-pipeline-state resource_lowering=argument-buffer-table texture_lowering=texture2d-sampler-argument".to_owned()
            },
            ExecutionResourceBinding {
                key: "shader_buffer".to_owned(),
                kind: ExecutionResourceKind::Buffer,
                capability_label: Some(domain_resource_capability_label(
                    "shader",
                    Some("metal.apple-silicon-gpu"),
                    "shader_buffer",
                    &ExecutionResourceKind::Buffer,
                )),
                value: "mock://bind/shader.buffer".to_owned()
            },
            ExecutionResourceBinding {
                key: "draw_handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    "shader",
                    Some("metal.apple-silicon-gpu"),
                    "draw_handle",
                    &ExecutionResourceKind::Handle,
                )),
                value: "mock://submit/draw.handle".to_owned()
            },
            ExecutionResourceBinding {
                key: "frame_target".to_owned(),
                kind: ExecutionResourceKind::Response,
                capability_label: Some(domain_resource_capability_label(
                    "shader",
                    Some("metal.apple-silicon-gpu"),
                    "frame_target",
                    &ExecutionResourceKind::Response,
                )),
                value: "mock://wait/frame.target".to_owned()
            }
        ]
    );

    let summary = trace.render_summary();
    assert!(summary.contains("event finalize role=Execute adapter=passive-adapter"));
    assert!(summary.contains(
        "resolved_resource shader_buffer kind=Buffer capability=cap.shader.metal_apple_silicon_gpu.buffer.shader_buffer value=mock://bind/shader.buffer"
    ));
    assert!(summary.contains(
        "resolved_resource frame_target kind=Response capability=cap.shader.metal_apple_silicon_gpu.frame.frame_target value=mock://wait/frame.target"
    ));
}
