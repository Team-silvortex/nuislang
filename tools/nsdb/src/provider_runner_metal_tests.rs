use super::parse_metal_runner_output;

#[cfg(target_os = "macos")]
use super::{
    execute_f32_argmax_input, execute_f32_bias_input, execute_gray8_invert,
    execute_gray8_threshold, execute_u32_canonical_input, execute_u32_copy_input,
};
#[cfg(target_os = "macos")]
use crate::provider_carrier_input::ProviderCarrierInput;
#[cfg(target_os = "macos")]
use crate::provider_runner_metal_render::execute_rgba8_render;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use yir_core::{
    ExecutionState, InstructionSemantics, Node, Operation, ProviderCompletionClockKind,
    ProviderCompletionRegistration, ProviderPhysicalCompletion, RegisteredMod, Resource, Value,
    YirResultFamily,
};

#[cfg(target_os = "macos")]
fn registered_shader_asset(asset_id: &str) -> (Vec<u8>, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../nustar-packages");
    let manifest = nuisc::registry::load_manifest_for_domain(&root, "shader").unwrap();
    let asset = nuisc::registry::code_asset_registration_by_id(&root, &manifest, asset_id)
        .unwrap()
        .expect("registered shader asset");
    (asset.bytes, asset.entry)
}

#[cfg(target_os = "macos")]
struct MetalRasterShaderMod {
    probe_input: PathBuf,
    emit_completion: bool,
}

#[cfg(target_os = "macos")]
impl RegisteredMod for MetalRasterShaderMod {
    fn module_name(&self) -> &'static str {
        "shader"
    }

    fn provider_completion_registration(
        &self,
        node: &Node,
    ) -> Option<ProviderCompletionRegistration> {
        yir_domain_shader::ShaderMod
            .provider_completion_registration(node)
            .map(|registration| {
                ProviderCompletionRegistration::physical_fence_required(
                    registration.family,
                    registration.clock_domain,
                )
            })
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        yir_domain_shader::ShaderMod.describe(node, resource)
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        let requires_completion = self.provider_completion_registration(node).is_some();
        let value = yir_domain_shader::ShaderMod.execute(node, resource, state)?;
        if requires_completion && self.emit_completion {
            let execution = if node.op.instruction == "draw_instanced" {
                let pass = match state.expect_value(&node.op.args[0])?.clone() {
                    Value::RenderPass(pass) => pass,
                    other => {
                        return Err(format!(
                            "Metal raster provider expected render pass, got {other}"
                        ))
                    }
                };
                let module = pass.shader_module.as_ref().ok_or_else(|| {
                    "Metal raster provider requires a shader module bound to the render pass"
                        .to_owned()
                })?;
                if module.language != "wgsl" {
                    return Err(format!(
                        "Metal raster provider cannot lower `{}` shader source",
                        module.language
                    ));
                }
                let lowered = nuisc::shader_msl_render_emitter::lower_canonical_inline_wgsl_render_for_profile(
                    &module.source,
                    &module.entry,
                    "metal.apple-silicon-gpu",
                )?;
                execute_rgba8_render(
                    &lowered.source,
                    &lowered.vertex_entry,
                    &lowered.fragment_entry,
                    pass.viewport.width.min(pass.target.width).max(1),
                    pass.viewport.height.min(pass.target.height).max(1),
                )?
            } else {
                execute_gray8_invert(&self.probe_input, 15)?
            };
            state.stage_provider_physical_completion(node, execution.physical_completion)?;
            if node.op.instruction == "draw_instanced" {
                let pass = match state.expect_value(&node.op.args[0])? {
                    Value::RenderPass(pass) => pass,
                    _ => unreachable!("validated render pass above"),
                };
                return yir_core::FrameSurface::from_rgba8(
                    pass.viewport.width.min(pass.target.width).max(1),
                    pass.viewport.height.min(pass.target.height).max(1),
                    execution.output_payload.as_bytes().to_vec(),
                )
                .map(Value::Frame);
            }
        }
        Ok(value)
    }
}

#[test]
fn parses_ready_metal_runner_output() {
    let execution = parse_metal_runner_output(
        "protocol=nuis-metal-gray8-provider-runner-v1\nstatus=ready\ndevice=Apple M2\noutput_bytes=4\ncompletion_contract=nuis-yir-provider-physical-completion-v1\ncompletion_status=fence-observed\ncompletion_target_clock_domain=shader.clock.frame.v1\ncompletion_source_clock_domain=apple.mach-continuous.v1\ncompletion_fence_source=metal.command-buffer.completed\ncompletion_source_clock=42\noutput_hex=0f0b0607\n",
    )
    .unwrap();

    assert_eq!(execution.contract, "nuis-metal-gray8-provider-runner-v1");
    assert_eq!(execution.status, "metal-command-buffer-completed");
    assert_eq!(execution.device, "Apple M2");
    assert_eq!(execution.output_payload.as_bytes(), [15, 11, 6, 7]);
    assert_eq!(execution.physical_completion.source_clock, 42);
    assert_eq!(
        execution.physical_completion.target_clock_domain,
        "shader.clock.frame.v1"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn executes_gray8_invert_on_the_system_metal_device() {
    let input = std::env::temp_dir().join(format!(
        "nuis-metal-gray8-input-{}-{}.bin",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&input, [0, 4, 9, 8]).unwrap();
    let first = execute_gray8_invert(&input, 15).expect("system Metal provider execution");
    let second = execute_gray8_invert(&input, 15).expect("second system Metal provider execution");
    let _ = std::fs::remove_file(input);

    assert_eq!(first.contract, "nuis-metal-gray8-provider-runner-v1");
    assert_eq!(first.status, "metal-command-buffer-completed");
    assert!(!first.device.is_empty());
    assert_eq!(first.output_payload.as_bytes(), [15, 11, 6, 7]);
    assert!(second.physical_completion.source_clock > first.physical_completion.source_clock);

    let source = Node {
        name: "pixelmagic_frame".to_owned(),
        resource: "shader0".to_owned(),
        op: Operation::parse("shader.const", vec!["1".to_owned()]).unwrap(),
    };
    let registration =
        ProviderCompletionRegistration::new(YirResultFamily::Shader, "shader.clock.frame.v1");
    let mut state = ExecutionState::default();
    state.begin_registered_provider_completion(&source).unwrap();
    state
        .stage_provider_physical_completion(&source, first.physical_completion.clone())
        .unwrap();
    let first_witness = state
        .finish_registered_provider_completion(registration, &source)
        .unwrap();
    state.begin_registered_provider_completion(&source).unwrap();
    state
        .stage_provider_physical_completion(&source, second.physical_completion.clone())
        .unwrap();
    let second_witness = state
        .finish_registered_provider_completion(registration, &source)
        .unwrap();
    assert_eq!(first_witness.completion_clock, 1);
    assert_eq!(second_witness.completion_clock, 2);
    assert_eq!(
        second_witness.clock_kind,
        ProviderCompletionClockKind::PhysicalFence
    );

    state.begin_registered_provider_completion(&source).unwrap();
    state
        .stage_provider_physical_completion(&source, first.physical_completion)
        .unwrap();
    let stale = state
        .finish_registered_provider_completion(registration, &source)
        .unwrap_err();
    assert!(stale.contains("is stale"));

    let rebound_clock = second
        .physical_completion
        .source_clock
        .checked_add(1)
        .unwrap();
    let rebound = ProviderPhysicalCompletion::new(
        "shader.clock.frame.v1",
        "apple.mach-continuous.v1",
        "metal.other-queue.completed",
        rebound_clock,
    )
    .unwrap();
    state.begin_registered_provider_completion(&source).unwrap();
    state
        .stage_provider_physical_completion(&source, rebound)
        .unwrap();
    let rebound = state
        .finish_registered_provider_completion(registration, &source)
        .unwrap_err();
    assert!(rebound.contains("changed its registered clock binding"));
}

#[cfg(target_os = "macos")]
#[test]
fn ns_nova_presents_a_registered_frame_after_real_metal_completion() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project_root = workspace_root.join("examples/projects/domains/ns_nova_showcase");
    let artifacts = nuisc::pipeline::compile_project(&project_root)
        .expect("NS Nova showcase should compile in memory");
    let input = std::env::temp_dir().join(format!(
        "nuis-nova-metal-completion-{}.bin",
        std::process::id()
    ));
    std::fs::write(&input, [0, 4, 9, 8]).unwrap();
    let mut registry = yir_verify::default_registry();
    registry.register(MetalRasterShaderMod {
        probe_input: input.clone(),
        emit_completion: true,
    });

    let trace = yir_exec::execute_module_with_registry(&artifacts.yir, &registry)
        .expect("NS Nova should execute through the registered Metal completion adapter");
    let begin = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.full_name() == "shader.begin_pass")
        .expect("NS Nova render pass");
    let draw = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.full_name() == "shader.draw_instanced")
        .expect("NS Nova draw submission");
    let begin_witness = &trace.provider_completion_witnesses[&begin.name];
    let draw_witness = &trace.provider_completion_witnesses[&draw.name];

    assert_eq!(
        begin_witness.clock_kind,
        ProviderCompletionClockKind::PhysicalFence
    );
    assert_eq!(
        draw_witness.clock_kind,
        ProviderCompletionClockKind::PhysicalFence
    );
    assert_eq!(begin_witness.completion_clock, 5);
    assert_eq!(draw_witness.completion_clock, 6);
    assert!(
        draw_witness.physical_source_clock.unwrap() > begin_witness.physical_source_clock.unwrap()
    );
    let event_tail = trace.events.iter().rev().take(8).collect::<Vec<_>>();
    assert_eq!(
        trace.presented_frames.len(),
        3,
        "NS Nova trace did not present its completed frame; event tail: {event_tail:#?}"
    );
    for frame in &trace.presented_frames {
        let rgba8 = frame
            .rgba8
            .as_ref()
            .expect("Metal-backed NS Nova presentation must retain RGBA8 pixels");
        assert_eq!(rgba8.len(), 160 * 120 * 4);
        assert!(rgba8.chunks_exact(4).all(|pixel| pixel[3] == 255));
        assert_ne!(&rgba8[..3], &rgba8[rgba8.len() - 4..rgba8.len() - 1]);
    }
    assert!(trace
        .events
        .iter()
        .any(|event| event.contains("effect cpu.present_frame")));
    let ppm = yir_runtime_host::render_trace_to_ppm_bytes(&trace, 1)
        .expect("only the physically completed NS Nova frame should be exported");
    assert!(ppm.starts_with(b"P6\n160 120\n255\n"));
    assert_eq!(ppm.len(), b"P6\n160 120\n255\n".len() + 160 * 120 * 3);
    let ppm_header_len = b"P6\n160 120\n255\n".len();
    assert_eq!(
        &ppm[ppm_header_len..ppm_header_len + 3],
        &trace
            .presented_frames
            .last()
            .unwrap()
            .rgba8
            .as_ref()
            .unwrap()[..3]
    );

    registry.register(MetalRasterShaderMod {
        probe_input: input.clone(),
        emit_completion: false,
    });
    let missing = yir_exec::execute_module_with_registry(&artifacts.yir, &registry)
        .expect_err("a physical Shader provider must not present without fence evidence");
    assert!(missing.contains("requires physical fence evidence"));
    let _ = std::fs::remove_file(input);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_gray8_threshold_on_the_system_metal_device() {
    let input = std::env::temp_dir().join(format!(
        "nuis-metal-gray8-threshold-input-{}-{}.bin",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&input, [0, 4, 9, 8]).unwrap();
    let execution =
        execute_gray8_threshold(&input, 8, 15).expect("system Metal threshold execution");
    let _ = std::fs::remove_file(input);

    assert_eq!(
        execution.contract,
        "nuis-metal-gray8-threshold-provider-runner-v1"
    );
    assert_eq!(execution.output_payload.as_bytes(), [0, 0, 15, 15]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_f32_bias_from_opaque_carrier_bytes() {
    let input = ProviderCarrierInput::OpaqueBytes {
        handle: "memory:metal-test".to_owned(),
        bytes: [10.0f32, 16.0, 22.0, 28.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect(),
    };
    let (source_path, entry) =
        registered_shader_source_path("shader.witsage.vector-bias.metal", "witsage-vector-bias");
    let execution =
        execute_f32_bias_input(&input, 1.0, &source_path, &entry).expect("opaque Metal input");
    let _ = std::fs::remove_file(source_path);
    let values = execution
        .output_payload
        .as_bytes()
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(values, [11.0, 17.0, 23.0, 29.0]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_f32_argmax_from_opaque_carrier_bytes() {
    let input = ProviderCarrierInput::OpaqueBytes {
        handle: "memory:metal-argmax-test".to_owned(),
        bytes: [0.0f32, 16.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect(),
    };
    let (source_path, entry) =
        registered_shader_source_path("shader.witsage.argmax.metal", "witsage-argmax");
    let execution =
        execute_f32_argmax_input(&input, &source_path, &entry).expect("opaque Metal argmax input");
    let _ = std::fs::remove_file(source_path);
    assert_eq!(
        u32::from_le_bytes(execution.output_payload.as_bytes().try_into().unwrap()),
        1
    );
}

#[cfg(target_os = "macos")]
#[test]
fn executes_generated_u32_copy_msl_from_opaque_carrier_bytes() {
    let execution = execute_registered_u32_msl("shader.metal.copy-u32.msl", "copy-u32");
    assert_eq!(execution.contract, "nuis-metal-u32-copy-provider-runner-v1");
    assert_eq!(u32_values(&execution.output_payload), [1, 8, 13, 21]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_generated_u32_add_msl_from_opaque_carrier_bytes() {
    let execution = execute_registered_u32_msl("shader.metal.add-u32.msl", "add-u32");
    assert_eq!(
        execution.contract,
        "nuis-metal-u32-canonical-provider-runner-v1"
    );
    assert_eq!(u32_values(&execution.output_payload), [2, 16, 26, 42]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_generated_u32_sub_msl_from_opaque_carrier_bytes() {
    let execution = execute_registered_u32_msl("shader.metal.sub-u32.msl", "sub-u32");
    assert_eq!(
        execution.contract,
        "nuis-metal-u32-canonical-provider-runner-v1"
    );
    assert_eq!(u32_values(&execution.output_payload), [0, 0, 0, 0]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_generated_u32_mul_msl_from_opaque_carrier_bytes() {
    let execution = execute_registered_u32_msl("shader.metal.mul-u32.msl", "mul-u32");
    assert_eq!(
        execution.contract,
        "nuis-metal-u32-canonical-provider-runner-v1"
    );
    assert_eq!(u32_values(&execution.output_payload), [1, 64, 169, 441]);
}

#[cfg(target_os = "macos")]
#[test]
fn executes_generated_u32_xor_msl_from_opaque_carrier_bytes() {
    let execution = execute_registered_u32_msl("shader.metal.xor-u32.msl", "xor-u32");
    assert_eq!(
        execution.contract,
        "nuis-metal-u32-canonical-provider-runner-v1"
    );
    assert_eq!(u32_values(&execution.output_payload), [0, 0, 0, 0]);
}

#[cfg(target_os = "macos")]
fn execute_registered_u32_msl(asset_id: &str, operation: &str) -> super::MetalProviderExecution {
    let (source_path, entry) =
        registered_shader_source_path(asset_id, &format!("metal-u32-{operation}"));
    let input = ProviderCarrierInput::OpaqueBytes {
        handle: format!("memory:metal-u32-{operation}-test"),
        bytes: [1u32, 8, 13, 21]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect(),
    };
    let execution = if operation == "copy-u32" {
        execute_u32_copy_input(&input, &source_path, &entry)
    } else {
        execute_u32_canonical_input(&input, &source_path, &entry, operation)
    }
    .expect("generated Metal u32 operation");
    let _ = std::fs::remove_file(source_path);
    execution
}

#[cfg(target_os = "macos")]
fn registered_shader_source_path(asset_id: &str, label: &str) -> (PathBuf, String) {
    let (metal_source, entry) = registered_shader_asset(asset_id);
    let source_path = std::env::temp_dir().join(format!(
        "nuis-{label}-source-{}-{}.metal",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::write(&source_path, metal_source).unwrap();
    (source_path, entry)
}

#[cfg(target_os = "macos")]
fn u32_values(
    payload: &crate::provider_output_carrier_registry::ProviderOutputPayload,
) -> Vec<u32> {
    payload
        .as_bytes()
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect()
}
