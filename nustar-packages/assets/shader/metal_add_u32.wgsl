binding(0, 0) var<storage, read> input_values: array<u32>;

binding(0, 1) var<storage, read_write> output_values: array<u32>;

stage compute(workgroup_size(1, 1, 1)) {
  fn nuis_metal_add_u32(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx: u32 = gid.x;
    output_values[idx] = input_values[idx] + input_values[idx];
  }
}
