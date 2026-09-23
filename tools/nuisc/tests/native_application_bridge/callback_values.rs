use super::*;

const SENTINEL: u64 = -700_i64 as u64;

fn source(slots: usize) -> String {
    let kinds = ["bool", "i32", "i64", "f32", "f64"];
    let fields = (0..slots)
        .map(|i| format!("f{i}: {}", kinds[i % 5]))
        .collect::<Vec<_>>()
        .join(", ");
    let params = (0..slots)
        .map(|i| format!("p{i}: {}", kinds[i % 5]))
        .collect::<Vec<_>>()
        .join(", ");
    // Construct in reverse order, independently of the declared transport order.
    let values = |advanced| {
        (0..slots)
            .rev()
            .map(|i| {
                if advanced && i % 5 == 2 {
                    format!("f{i}: p{i} + 1")
                } else {
                    format!("f{i}: p{i}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "mod cpu Main {{
      struct Payload {{ {fields} }} struct State {{ payload: Payload }}
      fn start({params}) -> State {{
        if p0 {{ return State {{ payload: Payload {{ {} }} }}; }}
        return State {{ payload: Payload {{ {} }} }};
      }}
      fn step(state: State) -> State {{ return state; }}
      fn stop(state: State) -> State {{ return state; }}
      fn main() -> i64 {{ return 0; }}
    }}",
        values(false),
        values(true)
    )
}

struct Probe {
    body: String,
    expected: Vec<u64>,
    next: usize,
    entries: u64,
    words: usize,
}

impl Probe {
    fn new(slots: usize) -> Self {
        let words = 2 * slots + 3;
        let bytes = words * 8 + 2;
        Self {
            body: format!("\ndefine i64 @nuis_yir_entry() {{\n  %memory = alloca [{bytes} x i8], align 8\n  %base = getelementptr i8, ptr %memory, i64 1\n  %tail = getelementptr i8, ptr %memory, i64 {}\n  store i8 91, ptr %memory\n  store i8 92, ptr %tail\n", bytes - 1),
            expected: Vec::new(), next: 0, entries: 0, words,
        }
    }

    fn store(&mut self, slot: usize, value: u64) {
        let id = self.next;
        self.next += 1;
        self.body.push_str(&format!("  %p{id} = getelementptr i64, ptr %base, i64 {slot}\n  store i64 {}, ptr %p{id}, align 1\n", value as i64));
    }

    fn call(
        &mut self,
        symbol: &str,
        input: &[u64],
        output: &[u64],
        from: usize,
        to: usize,
        failure: Option<&str>,
    ) {
        let slots = input.len();
        let mut memory = vec![SENTINEL; self.words];
        memory[from..from + slots].copy_from_slice(input);
        let mut status = 0;
        let mut argc = slots as i64;
        let mut outc = argc;
        match failure {
            Some("bool") => {
                memory[from] = 2;
                status = 2;
            }
            Some("i32") => {
                memory[from + 1] = u32::MAX as u64;
                status = 2;
            }
            Some("f32") => {
                memory[from + 3] = 1 << 32;
                status = 2;
            }
            Some("argc") => {
                argc = -1;
                status = 1;
            }
            Some("outc") => {
                outc -= 1;
                status = 1;
            }
            Some("null-in" | "null-out") => {
                status = 1;
            }
            None => {}
            _ => unreachable!(),
        }
        for (index, value) in memory.iter().copied().enumerate() {
            self.store(index, value);
        }
        let id = self.next;
        self.next += 1;
        self.body.push_str(&format!("  %in{id} = getelementptr i64, ptr %base, i64 {from}\n  %out{id} = getelementptr i64, ptr %base, i64 {to}\n"));
        let args = if failure == Some("null-in") {
            "null".to_owned()
        } else {
            format!("%in{id}")
        };
        let out = if failure == Some("null-out") {
            "null".to_owned()
        } else {
            format!("%out{id}")
        };
        self.body.push_str(&format!("  %status{id} = call i32 @{symbol}(ptr {args}, i64 {argc}, ptr {out}, i64 {outc})\n  %wide{id} = zext i32 %status{id} to i64\n  call void @nuis_debug_print_i64(i64 %wide{id})\n"));
        self.expected.push(status);
        if status == 0 {
            memory[to..to + slots].copy_from_slice(output);
            self.entries += 1;
        }
        // Check the entire backing region, not only output slots, on every call.
        for (slot, expected) in memory.iter().enumerate() {
            self.body.push_str(&format!("  %r{id}_{slot} = getelementptr i64, ptr %base, i64 {slot}\n  %v{id}_{slot} = load i64, ptr %r{id}_{slot}, align 1\n  call void @nuis_debug_print_i64(i64 %v{id}_{slot})\n"));
            self.expected.push(*expected);
        }
        for (name, expected) in [
            ("callback_entries", self.entries),
            ("probe_allocs", 0),
            ("probe_drops", 0),
        ] {
            self.body.push_str(&format!("  %{name}{id} = load i64, ptr @{name}\n  call void @nuis_debug_print_i64(i64 %{name}{id})\n"));
            self.expected.push(expected);
        }
        for (edge, expected) in [("memory", 91), ("tail", 92)] {
            self.body.push_str(&format!("  %{edge}{id} = load i8, ptr %{edge}\n  %{edge}_wide{id} = zext i8 %{edge}{id} to i64\n  call void @nuis_debug_print_i64(i64 %{edge}_wide{id})\n"));
            self.expected.push(expected);
        }
    }
}

#[test]
fn callback_values_preserve_nested_bits_overlap_and_failure_sentinels_without_allocating() {
    for slots in [1, 6, 64] {
        let source = source(slots);
        let project = Project::with_source(&source);
        let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        compiled.yir.nodes.reverse();
        compiled.yir.functions.reverse();
        for function in &mut compiled.yir.functions {
            function.body_nodes.reverse();
        }
        let bridge = emit_registered(&compiled.yir, "counter").unwrap();
        let mut llvm = bridge.llvm_ir.replacen(
            "define i64 @nuis_yir_entry()",
            "define i64 @unused_native_entry()",
            1,
        );
        assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
        for export in &bridge.callbacks {
            let start = aggregate_values::definition(&llvm, &export.function);
            let insertion = start + llvm[start..].find("{\n").unwrap() + 2;
            llvm.insert_str(insertion, "  %entries = load i64, ptr @callback_entries\n  %next_entries = add i64 %entries, 1\n  store i64 %next_entries, ptr @callback_entries\n");
            let start = llvm
                .find(&format!("define i32 @{}(", export.symbol))
                .unwrap();
            let body = &llvm[start..start + llvm[start..].find("\n}\n").unwrap()];
            let returned = body.find("%returned = call [").unwrap();
            let stored = body.find("store i64 %result0").unwrap();
            assert!(body.rfind(" = load i64, ptr %input_ptr").unwrap() < returned);
            assert!(returned < body.rfind(" = extractvalue [").unwrap());
            assert!(body.rfind(" = extractvalue [").unwrap() < stored);
        }
        llvm.push_str("\n@callback_entries = internal global i64 0\n");
        llvm.push_str(multi_execution::ALLOCATION_PROBE);
        let mut probe = Probe::new(slots);
        let registry = yir_verify::default_registry();
        for flag in [0, 1] {
            for (gain, scale) in [
                (1.5_f32.to_bits(), (-2.25_f64).to_bits()),
                (0x8000_0000, 0x8000_0000_0000_0000),
                (0x7fc0_1234, 0x7ff8_0000_0000_4321),
            ] {
                let input = (0..slots)
                    .map(|i| match i % 5 {
                        0 => flag,
                        1 => (-17 - i as i64) as u64,
                        2 => i64::MAX as u64 - i as u64,
                        3 => u64::from(gain),
                        _ => scale,
                    })
                    .collect::<Vec<_>>();
                let mut advanced = input.clone();
                if flag == 0 {
                    for i in (2..slots).step_by(5) {
                        advanced[i] += 1;
                    }
                }
                let values = bridge.callbacks[0]
                    .arguments
                    .iter()
                    .zip(&input)
                    .map(|(kind, word)| kind.unpack(*word).unwrap())
                    .collect();
                let (mut reference, _) = ApplicationSession::open_registered(
                    &compiled.yir,
                    &registry,
                    "counter",
                    values,
                )
                .unwrap();
                assert_eq!(state_words(reference.state()), advanced);
                reference.event(vec![]).unwrap();
                assert_eq!(state_words(reference.state()), advanced);
                reference.close(vec![]).unwrap().unwrap();
                assert_eq!(state_words(reference.state()), advanced);
                reference.completion_status().unwrap();
                for (role, export) in bridge.callbacks.iter().enumerate() {
                    for (from, to) in [(1, 1), (1, 2), (2, 1), (1, slots + 2)] {
                        let output = if role == 0 { &advanced } else { &input };
                        probe.call(&export.symbol, &input, output, from, to, None);
                        if flag == 0 && gain == 0x7fc0_1234 {
                            for failure in
                                ["argc", "outc", "null-in", "null-out", "bool", "i32", "f32"]
                            {
                                if slots == 1 && matches!(failure, "i32" | "f32") {
                                    continue;
                                }
                                probe.call(&export.symbol, &input, output, from, to, Some(failure));
                            }
                        }
                    }
                }
            }
        }
        probe.body.push_str("  ret i64 0\n}\n");
        llvm.push_str(&probe.body);
        let artifact = nuisc::aot::write_and_link_with_source(
            &project.0.join("main.ns"),
            &project.0.join("out"),
            &source,
            nuisc::aot::AotCompileProgram {
                ast: &compiled.ast,
                nir: &compiled.nir,
                yir: &compiled.yir,
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
            .map(|v| v.parse::<i64>().unwrap() as u64)
            .collect::<Vec<_>>();
        assert_eq!(actual, probe.expected, "slots={slots}");
    }
}
