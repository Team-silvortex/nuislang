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
  %1 = add i64 0, 8
  %2 = add i64 0, 2
  %3 = add i64 0, 16
  %4 = add i64 0, 0
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  %5 = add i64 0, 1
  ; deferred lowering for data.immutable_window on fabric0 (data.fabric)
  ; deferred lowering for data.handle_table on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  %6 = add i64 0, 1
  ; deferred lowering for data.copy_window on fabric0 (data.fabric)
  ; deferred lowering for data.bind_core on fabric0 (data.fabric)
  ; deferred lowering for data.marker on fabric0 (data.fabric)
  ; deferred lowering for cpu.instantiate_unit on cpu0 (cpu.arm64)
  ; deferred lowering for cpu.instantiate_unit on cpu0 (cpu.arm64)
  %7 = add i64 0, 7
  ; deferred lowering for cpu.bind_core on cpu0 (cpu.arm64)
  %8 = add i64 %7, %2
  %9 = add i64 %8, %1
  %10 = add i64 %9, %3
  ; deferred lowering for data.immutable_window on fabric0 (data.fabric)
  ; deferred lowering for data.output_pipe on fabric0 (data.fabric)
  ; deferred lowering for data.input_pipe on fabric0 (data.fabric)
  ; deferred lowering for kernel.target_config on kernel0 (kernel.apple)
  ; deferred lowering for data.copy_window on fabric0 (data.fabric)
  ; deferred lowering for data.output_pipe on fabric0 (data.fabric)
  ; deferred lowering for data.input_pipe on fabric0 (data.fabric)
  ; deferred lowering for cpu.print `data_input_pipe_21` because its input is produced outside the current CPU LLVM slice
  ret i64 %10
}
