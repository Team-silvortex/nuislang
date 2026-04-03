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

declare i64 @host_color_bias(i64)
declare i64 @host_speed_curve(i64)
declare i64 @host_radius_curve(i64)
declare i64 @host_mix_tick(i64, i64)

declare i64 @HostRenderCurves__color_bias(i64)
declare i64 @HostRenderCurves__speed_curve(i64)
declare i64 @HostRenderCurves__radius_curve(i64)
declare i64 @HostRenderCurves__mix_tick(i64, i64)
declare i64 @HostMath__speed_curve(i64)

define i64 @nuis_yir_entry() {
  %1 = add i64 0, 4
  ; deferred lowering for shader.viewport on shader0 (shader.render)
  ; deferred lowering for shader.target on shader0 (shader.render)
  ; deferred lowering for shader.pipeline on shader0 (shader.render)
  ; deferred lowering for shader.begin_pass on shader0 (shader.render)
  %2 = add i64 0, 1
  %3 = add i64 0, 0
  %4 = add i64 0, 1
  ; deferred lowering for data.immutable_window on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.handle_table on fabric0 (data.fabric)
  %5 = add i64 0, 1
  ; deferred lowering for data.copy_window on fabric0 (data.fabric)
  ; deferred lowering for data.bind_core on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for cpu.instantiate_unit on cpu0 (cpu.arm64)
  ; deferred lowering for cpu.instantiate_unit on cpu0 (cpu.arm64)
  %6 = add i64 0, 2
  %7 = add i64 0, 96
  %8 = add i64 0, 10
  %9 = add i64 0, 104
  ; deferred lowering for cpu.window on cpu0 (cpu.arm64)
  ; deferred lowering for cpu.tick_i64 on cpu0 (cpu.arm64)
  ; static AOT lowering freezes cpu.input_i64 `radius_knob` to its default value
  %10 = add i64 0, 18
  ; static AOT lowering freezes cpu.input_i64 `speed_knob` to its default value
  %11 = add i64 0, 6
  %12 = mul i64 %11, %6
  ; static AOT lowering freezes cpu.input_i64 `color_knob` to its default value
  %13 = add i64 0, 24
  ; deferred lowering for cpu.bind_core on cpu0 (cpu.arm64)
  %14 = add i64 %7, %10
  %15 = add i64 %14, %1
  %16 = call i64 @HostRenderCurves__radius_curve(i64 %15)
  ; deferred lowering for cpu.extern_call_i64 `cpu_extern_call_22` because one or more inputs are outside the current CPU LLVM slice
  %17 = add i64 %12, %8
  %18 = add i64 %17, %2
  %19 = call i64 @HostRenderCurves__speed_curve(i64 %18)
  ; deferred lowering for cpu.extern_call_i64 `cpu_extern_call_18` because one or more inputs are outside the current CPU LLVM slice
  %20 = add i64 %9, %13
  %21 = call i64 @HostRenderCurves__color_bias(i64 %20)
  ; deferred lowering for cpu.struct `struct_23` because field `speed` comes from outside the current CPU LLVM slice
  ; deferred lowering for data.output_pipe on fabric0 (data.fabric)
  ; deferred lowering for data.input_pipe on fabric0 (data.fabric)
  ; deferred lowering for shader.draw_instanced on shader0 (shader.render)
  ; deferred lowering for data.output_pipe on fabric0 (data.fabric)
  ; deferred lowering for data.input_pipe on fabric0 (data.fabric)
  ; deferred lowering for cpu.present_frame on cpu0 (cpu.arm64)
  ret i64 %21
}
