use super::*;

fn source(ty: &str) -> String {
    let tiny = if ty == "f32" {
        format!("{:.46}", f32::from_bits(1))
    } else {
        format!("{:.324}", f64::from_bits(1))
    };
    format!(
        "mod cpu Main {{
        struct State {{ negative: {ty}, positive: {ty}, triple: {ty},
            finite: {ty}, double: {ty}, computed: {ty}, rounded: {ty}, tiny: {ty} }}
        @noinline fn sample(value: {ty}) -> {ty} {{ return value; }}
        @noinline fn relay(state: State) -> State {{ return state; }}
        fn start() -> State {{
            let negative: {ty} = -0.0;
            const positive: {ty} = -(-0.0);
            return State {{ negative: negative, positive: positive,
                triple: -(-(-0.0)), finite: -1.25,
                double: -(-2.5), computed: -sample(1.25),
                rounded: -0.1, tiny: -{tiny} }};
        }}
        fn step(state: State) -> State {{
            let carry = state; let i = 0;
            while i < 1 {{ let i = i + 1; let carry = relay(carry); }}
            return carry;
        }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

fn check(ty: &str) {
    let source = source(ty);
    // The oracle is host bit arithmetic, not numeric float equality or YIR output.
    let words = if ty == "f32" {
        [
            -0.0_f32,
            0.0,
            -0.0,
            -1.25,
            2.5,
            -1.25,
            -0.1,
            -f32::from_bits(1),
        ]
        .map(|value| u64::from(value.to_bits()))
        .to_vec()
    } else {
        [
            -0.0_f64,
            0.0,
            -0.0,
            -1.25,
            2.5,
            -1.25,
            -0.1,
            -f64::from_bits(1),
        ]
        .map(f64::to_bits)
        .to_vec()
    };
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let registry = yir_verify::default_registry();
    for reversed in [false, true] {
        let mut module = compiled.yir.clone();
        if reversed {
            module.nodes.reverse();
            module.functions.reverse();
            for function in &mut module.functions {
                function.body_nodes.reverse();
            }
        }
        let (mut session, _) =
            ApplicationSession::open_registered(&module, &registry, "counter", vec![]).unwrap();
        assert_eq!(state_words(session.state()), words);
        session.event(vec![]).unwrap();
        assert_eq!(state_words(session.state()), words);
        session.close(vec![]).unwrap().unwrap();
        assert_eq!(state_words(session.state()), words);
        session.completion_status().unwrap();

        let bridge = emit_registered(&module, "counter").unwrap();
        assert_eq!(bridge.state_fields.len(), words.len());
        let mut llvm = bridge.llvm_ir.replacen(
            "define i64 @nuis_yir_entry()",
            "define i64 @unused_native_entry()",
            1,
        );
        llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %words = alloca [8 x i64], align 8\n");
        let mut expected = Vec::new();
        for (role, export) in bridge.callbacks.iter().enumerate() {
            let count = export.arguments.len();
            llvm.push_str(&format!("  %status{role} = call i32 @{}(ptr %words, i64 {count}, ptr %words, i64 8)\n  %wide{role} = zext i32 %status{role} to i64\n  call void @nuis_debug_print_i64(i64 %wide{role})\n", export.symbol));
            expected.push(0);
            for (slot, word) in words.iter().enumerate() {
                llvm.push_str(&format!("  %p{role}_{slot} = getelementptr i64, ptr %words, i64 {slot}\n  %v{role}_{slot} = load volatile i64, ptr %p{role}_{slot}, align 8\n  call void @nuis_debug_print_i64(i64 %v{role}_{slot})\n"));
                expected.push(*word as i64);
            }
        }
        llvm.push_str("  ret i64 0\n}\n");
        let artifact = nuisc::aot::write_and_link_with_source(
            &project.0.join("main.ns"),
            &project.0.join(format!("out-{reversed}")),
            &source,
            nuisc::aot::AotCompileProgram {
                ast: &compiled.ast,
                nir: &compiled.nir,
                yir: &module,
                llvm_ir: Some(&llvm),
            },
            &nuisc::aot::host_cpu_build_target(),
        )
        .unwrap();
        let run = dynamic_loop_guard::run_bounded(
            std::path::Path::new(&artifact.binary_path),
            &project.0,
        );
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let actual = String::from_utf8(run.stdout)
            .unwrap()
            .lines()
            .map(|line| line.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{ty} reversed={reversed}");
    }
}

#[test]
fn unary_float_literals_f32_preserve_native_and_yir_bits() {
    check("f32");
}

#[test]
fn unary_float_literals_f64_preserve_native_and_yir_bits() {
    check("f64");
}
