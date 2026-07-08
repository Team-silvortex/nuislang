use super::*;

#[test]
fn lowers_data_result_primitives_into_data_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let result: DataResult<Pipe<i64>> = data_result(data_output_pipe(7));
            let moved: bool = data_moved(result);
            return data_value(result);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "is_moved"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "value"));
}

#[test]
fn lowers_shader_result_primitives_into_shader_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let result: ShaderResult<Pass> = shader_result(shader_begin_pass(
              shader_target("rgba8", 8, 8),
              shader_pipeline("flat", "triangle"),
              shader_viewport(8, 8)
            ));
            let ready: bool = shader_pass_ready(result);
            return 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "is_pass_ready"));
}

#[test]
fn lowers_shader_texture_sampling_into_shader_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let texture: Texture = shader_texture2d("r8", 2, 2, "1,2,3,4");
            let sampler: Sampler = shader_sampler("nearest", "clamp");
            return shader_sample_nearest(texture, sampler, 1, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "texture2d"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "sampler"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "sample_nearest"));
}

#[test]
fn lowers_shader_uv_sampling_into_shader_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let texture: Texture = shader_texture2d("r8", 2, 2, "1,2,3,4");
            let sampler: Sampler = shader_sampler("linear", "clamp");
            let uv: UV = shader_uv(512, 256);
            return shader_sample_uv_linear(texture, sampler, uv);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "uv"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "sample_uv_linear"));
}

#[test]
fn lowers_shader_binding_set_into_shader_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pipeline: Pipeline = shader_pipeline("blit", "triangle-strip");
            let texture: Texture = shader_texture2d("r8", 2, 2, "1,2,3,4");
            let sampler: Sampler = shader_sampler("linear", "clamp");
            let texture_binding: Binding = shader_texture_binding(0, texture);
            let sampler_binding: Binding = shader_sampler_binding(1, sampler);
            let bindings: BindingSet =
              shader_bind_set(pipeline, texture_binding, sampler_binding);
            print(bindings);
            return 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "texture_binding"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "sampler_binding"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "bind_set"));
}

#[test]
fn lowers_shader_buffer_bindings_into_shader_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pipeline: Pipeline = shader_pipeline("blit", "triangle-strip");
            let uniform_buffer: ref Buffer = alloc_buffer(64, 0);
            let storage_buffer: ref Buffer = alloc_buffer(128, 1);
            let uniform_binding: Binding = shader_uniform_binding(2, uniform_buffer);
            let storage_binding: Binding = shader_storage_binding(3, storage_buffer);
            let bindings: BindingSet =
              shader_bind_set(pipeline, uniform_binding, storage_binding);
            print(bindings);
            return 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "uniform_binding"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "storage_binding"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "bind_set"));
}

#[test]
fn lowers_shader_buffer_binding_layouts_into_shader_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pipeline: Pipeline = shader_pipeline("blit", "triangle-strip");
            let uniform_buffer: ref Buffer = alloc_buffer(64, 0);
            let storage_buffer: ref Buffer = alloc_buffer(128, 1);
            let uniform_binding: Binding =
              shader_uniform_binding_layout(2, "std140", uniform_buffer);
            let storage_binding: Binding =
              shader_storage_binding_layout(3, "std430", storage_buffer);
            let bindings: BindingSet =
              shader_bind_set(pipeline, uniform_binding, storage_binding);
            print(bindings);
            return 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir.nodes.iter().any(|node| {
        node.op.module == "shader"
            && node.op.instruction == "uniform_binding"
            && node.op.args.len() == 3
            && node.op.args[0] == "2"
            && node.op.args[1] == "std140"
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.op.module == "shader"
            && node.op.instruction == "storage_binding"
            && node.op.args.len() == 3
            && node.op.args[0] == "3"
            && node.op.args[1] == "std430"
    }));
}

#[test]
fn lowers_shader_packet_bindings_into_shader_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let pipeline: Pipeline = shader_pipeline("blit", "triangle-strip");
            let packet: NovaPanelPacket = nova_panel_packet(1, 2, 3, 4, 5, 6);
            let uniform_binding: Binding = shader_packet_uniform_binding(4, packet);
            let storage_binding: Binding = shader_packet_storage_binding(5, packet);
            let bindings: BindingSet =
              shader_bind_set(pipeline, uniform_binding, storage_binding);
            print(bindings);
            return 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir.nodes.iter().any(|node| {
        node.op.module == "shader"
            && node.op.instruction == "uniform_binding"
            && node.op.args.len() == 4
            && node.op.args[0] == "4"
            && node.op.args[1] == "std140"
            && node.op.args[2] == "shader.profile.packet.nova.v1"
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.op.module == "shader"
            && node.op.instruction == "storage_binding"
            && node.op.args.len() == 4
            && node.op.args[0] == "5"
            && node.op.args[1] == "std430"
            && node.op.args[2] == "shader.profile.packet.nova.v1"
    }));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "bind_set"));
}

#[test]
fn lowers_kernel_result_primitives_into_kernel_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let result: KernelResult<i64> = kernel_result(kernel_profile_queue_depth("KernelUnit"));
            let ready: bool = kernel_config_ready(result);
            return kernel_value(result);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "value"));
}

#[test]
fn lowers_network_result_primitives_into_network_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let result: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let send_ready: bool = network_send_ready(result);
            let recv_ready: bool = network_recv_ready(result);
            let config_ready: bool = network_config_ready(result);
            return network_value(result);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_send_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_recv_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "value"));
}
