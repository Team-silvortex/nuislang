use super::*;

#[test]
fn native_bridge_accepts_the_slot_bound_and_rejects_larger_signatures() {
    let project = Project::new();
    for count in [64, 65] {
        let fields = (0..count)
            .map(|index| format!("v{index}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (0..count)
            .map(|index| format!("v{index}: seed"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!(
            "mod cpu Main {{
            struct State {{ {fields} }}
            fn start(seed: i64) -> State {{ return State {{ {values} }}; }}
            fn step(state: State) -> State {{ return state; }}
            fn stop(state: State) -> State {{ return state; }}
            fn main() -> i64 {{ return 0; }}
        }}"
        );
        fs::write(project.0.join("main.ns"), source).unwrap();
        let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        let result = emit_registered(&module, "counter");
        if count == 64 {
            assert_eq!(result.unwrap().state_fields.len(), count);
        } else {
            assert!(result.unwrap_err().contains("bounds"));
        }
    }
}

#[test]
fn zero_argument_native_open_accepts_null_input_without_accessing_it() {
    let project = Project::new();
    let source = "mod cpu Main {
        struct State { value: i64 }
        fn start() -> State { return State { value: 42 }; }
        fn step(state: State) -> State { return state; }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }";
    fs::write(project.0.join("main.ns"), source).unwrap();
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    assert!(bridge.callbacks[0].arguments.is_empty());
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    llvm.push_str(&format!(
        "\ndefine i64 @nuis_yir_entry() {{
        %out = alloca i64, align 8
        store i64 -1, ptr %out, align 8
        %status = call i32 @{}(ptr null, i64 0, ptr %out, i64 1)
        %wide = zext i32 %status to i64
        call void @nuis_debug_print_i64(i64 %wide)
        %value = load i64, ptr %out, align 8
        call void @nuis_debug_print_i64(i64 %value)
        ret i64 0
    }}\n",
        bridge.callbacks[0].symbol
    ));
    let artifact = nuisc::aot::write_and_link_with_source(
        &project.0.join("main.ns"),
        &project.0.join("out-zero"),
        source,
        nuisc::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &compiled.nir,
            yir: &compiled.yir,
            llvm_ir: Some(&llvm),
        },
        &nuisc::aot::host_cpu_build_target(),
    )
    .unwrap();
    let output = Command::new(artifact.binary_path).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "0\n42\n");
}
