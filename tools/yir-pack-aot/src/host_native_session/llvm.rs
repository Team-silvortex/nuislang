use super::NativeSessionBridge;

fn bytes(name: &str, value: &str) -> String {
    let bytes = value
        .bytes()
        .map(|byte| format!("\\{byte:02X}"))
        .collect::<String>();
    format!(
        "@{name} = private constant [{} x i8] c\"{bytes}\"\n",
        value.len()
    )
}

fn getter(bridge: &NativeSessionBridge) -> String {
    format!("{}_binding", bridge.callbacks[0].symbol)
}

pub(super) fn callbacks(bridge: &NativeSessionBridge, source: &str) -> String {
    let mut llvm = bridge.llvm_ir.clone();
    llvm.push_str(&bytes("native_compiled_yir", source));
    llvm.push_str(&bytes("native_session_id", &bridge.session_id));
    llvm.push_str(&bytes("native_state_layout", bridge.state_layout.source()));
    // Mirrors NativeSessionDescriptorV1: fixed u64 lengths and static pointers.
    llvm.push_str(&format!(
        "@native_session_binding = private constant {{ i64, ptr, i64, ptr, i64, ptr, i64, ptr, ptr, ptr }} {{ i64 1, ptr @native_compiled_yir, i64 {}, ptr @native_session_id, i64 {}, ptr @native_state_layout, i64 {}, ptr @{}, ptr @{}, ptr @{} }}\n\ndefine ptr @{}() {{\nentry:\n  ret ptr @native_session_binding\n}}\n",
        source.len(), bridge.session_id.len(), bridge.state_layout.source().len(), bridge.callbacks[0].symbol, bridge.callbacks[1].symbol, bridge.callbacks[2].symbol, getter(bridge)
    ));
    llvm
}

pub(super) fn entry(bridge: &NativeSessionBridge, source: &str) -> String {
    format!(
        "{}{}\ndeclare ptr @{}()\ndeclare i32 @nuis_native_application_script_main(ptr, i64, ptr, i32, ptr)\n\ndefine i32 @main(i32 %argc, ptr %argv) {{\nentry:\n  %binding = call ptr @{}()\n  %status = call i32 @nuis_native_application_script_main(ptr @native_expected_yir, i64 {}, ptr %binding, i32 %argc, ptr %argv)\n  ret i32 %status\n}}\n",
        bytes("native_expected_yir", source), literal_print_runtime(!bridge.literal_print_sites.is_empty()), getter(bridge), getter(bridge), source.len()
    )
}

fn literal_print_runtime(enabled: bool) -> String {
    if !enabled {
        return String::new();
    }
    // Existing scalar print ABI, emitted in the host unit, not new effect authority.
    format!("{}declare i32 @printf(ptr, ...)\n\ndefine void @nuis_debug_print_i64(i64 %value) {{\nentry:\n  %written = call i32 (ptr, ...) @printf(ptr @native_literal_print_format, i64 %value)\n  ret void\n}}\n",
        bytes("native_literal_print_format", "%lld\n\0"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_literal_print_host_runtime_is_opt_in_and_i64_only() {
        assert!(literal_print_runtime(false).is_empty());
        let llvm = literal_print_runtime(true);
        assert!(llvm.contains("[6 x i8]"));
        assert!(llvm.contains("\\25\\6C\\6C\\64\\0A\\00"));
        assert!(llvm.contains("define void @nuis_debug_print_i64(i64 %value)"));
        assert!(llvm.contains("@printf(ptr @native_literal_print_format, i64 %value)"));
        for forbidden in [
            "print_i32",
            "print_bool",
            "print_f32",
            "print_f64",
            "malloc",
            "fflush",
        ] {
            assert!(!llvm.contains(forbidden), "{forbidden}");
        }
    }
}
