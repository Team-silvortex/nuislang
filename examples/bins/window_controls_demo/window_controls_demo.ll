; yir version: 0.1

%cpu.node = type { i64, ptr }
declare ptr @malloc(i64)
declare void @free(ptr)
declare i32 @puts(ptr)
declare void @nuis_debug_print_bool(i32)
declare void @nuis_debug_print_i32(i32)
declare void @nuis_debug_print_i64(i64)

declare void @nuis_debug_print_f32(float)
declare void @nuis_debug_print_f64(double)

define i64 @nuis_yir_entry() {
  ; deferred lowering for shader.target on shader0 (shader.render)
  %1 = add i64 0, 2
  ; deferred lowering for shader.viewport on shader0 (shader.render)
  ; deferred lowering for cpu.bind_core on cpu0 (cpu.arm64)
  ; deferred lowering for shader.pipeline on shader0 (shader.render)
  ; deferred lowering for shader.begin_pass on shader0 (shader.render)
  ; deferred lowering for cpu.tick_i64 on cpu0 (cpu.arm64)
  ; deferred lowering for data.bind_core on fabric0 (data.fabric)
  ; deferred lowering for data.handle_table on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for cpu.target_config on cpu0 (cpu.arm64)
  ; deferred lowering for cpu.window on cpu0 (cpu.arm64)
  ; static AOT lowering freezes cpu.input_i64 `speed_knob` to its default value
  %2 = add i64 0, 6
  ; static AOT lowering freezes cpu.input_i64 `radius_knob` to its default value
  %3 = add i64 0, 18
  ; static AOT lowering freezes cpu.input_i64 `color_knob` to its default value
  %4 = add i64 0, 24
  %5 = add i64 0, 10
  %6 = add i64 0, 96
  %7 = mul i64 %2, %1
  %8 = add i64 %7, %5
  ; deferred lowering for cpu.add `panel_speed` because one or more inputs are outside the current CPU LLVM slice
  %9 = add i64 %6, %3
  ; deferred lowering for cpu.add `panel_radius` because one or more inputs are outside the current CPU LLVM slice
  %10 = add i64 0, 104
  %11 = add i64 %10, %4
  ; deferred lowering for cpu.struct `render_controls` because field `speed` comes from outside the current CPU LLVM slice
  ; deferred lowering for data.output_pipe on fabric0 (data.fabric)
  ; deferred lowering for data.input_pipe on fabric0 (data.fabric)
  ; deferred lowering for shader.draw_instanced on shader0 (shader.render)
  ; deferred lowering for data.output_pipe on fabric0 (data.fabric)
  ; deferred lowering for data.input_pipe on fabric0 (data.fabric)
  ; deferred lowering for cpu.present_frame on cpu0 (cpu.arm64)
  ret i64 %11
}
