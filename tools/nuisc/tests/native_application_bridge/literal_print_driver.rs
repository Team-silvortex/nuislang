use super::*;

const PROBES: [&str; 6] = ["choose", "observe", "finish", "start", "step", "stop"];
const CANARY: i64 = -900_123;

pub(super) struct Driver {
    pub llvm: String,
    pub expected: Vec<i64>,
    next: usize,
}

impl Driver {
    pub fn new(bridge: &NativeSessionBridge) -> Self {
        let mut llvm = bridge.llvm_ir.replacen(
            "define i64 @nuis_yir_entry()",
            "define i64 @unused_native_entry()",
            1,
        );
        for name in PROBES {
            let start = aggregate_values::definition(&llvm, name);
            let offset = llvm[start..]
                .find(", ptr %nuis_helper_entries, align 8\n")
                .unwrap();
            let insertion = start + offset + ", ptr %nuis_helper_entries, align 8\n".len();
            // Observe admitted source entries, without replacing computation or fuel.
            llvm.insert_str(insertion, &format!("  %effect_probe = load volatile i64, ptr @{name}_effect_probe\n  %effect_next = add i64 %effect_probe, 1\n  store volatile i64 %effect_next, ptr @{name}_effect_probe\n"));
            llvm.push_str(&format!(
                "\n@{name}_effect_probe = internal global i64 0, align 8\n"
            ));
        }
        llvm = llvm.replace(
            "call void @nuis_debug_print_i64(",
            "call void @effect_source_print(",
        );
        if !llvm.contains("declare i32 @fflush(") {
            llvm.push_str("\ndeclare i32 @fflush(ptr)\n");
        }
        llvm.push_str("\ndefine void @effect_source_print(i64 %value) {\n  call void @nuis_debug_print_i64(i64 %value)\n  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\n");
        llvm.push_str("\n@effect_out = internal global [9 x i64] zeroinitializer, align 8\n");
        llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %storage = alloca [11 x i64], align 8\n  %args = getelementptr i64, ptr %storage, i64 1\n");
        Self {
            llvm,
            expected: Vec::new(),
            next: 0,
        }
    }

    pub fn store(&mut self, base: &str, slot: usize, value: u64) {
        let id = self.next;
        self.next += 1;
        self.llvm.push_str(&format!("  %write{id} = getelementptr i64, ptr {base}, i64 {slot}\n  store volatile i64 {}, ptr %write{id}, align 8\n", value as i64));
    }

    pub fn read(&mut self, base: &str, slot: usize, expected: i64) {
        let id = self.next;
        self.next += 1;
        self.llvm.push_str(&format!("  %read{id} = getelementptr i64, ptr {base}, i64 {slot}\n  %word{id} = load volatile i64, ptr %read{id}, align 8\n  call void @nuis_debug_print_i64(i64 %word{id})\n"));
        self.expected.push(expected);
    }

    fn reset(&mut self) {
        for name in PROBES {
            self.llvm.push_str(&format!(
                "  store volatile i64 0, ptr @{name}_effect_probe\n"
            ));
        }
        self.store("%storage", 0, CANARY as u64);
        self.store("%storage", 10, CANARY as u64);
    }

    pub fn call(
        &mut self,
        symbol: &str,
        args: &str,
        count: i64,
        out: &str,
        slots: i64,
        status: i64,
    ) {
        let id = self.next;
        self.next += 1;
        self.llvm.push_str(&format!("  %status{id} = call i32 @{symbol}(ptr {args}, i64 {count}, ptr {out}, i64 {slots})\n  %wide{id} = sext i32 %status{id} to i64\n  call void @nuis_debug_print_i64(i64 %wide{id})\n"));
        self.expected.push(status);
    }

    pub fn valid(
        &mut self,
        bridge: &NativeSessionBridge,
        input: [u64; 8],
        state: &[u64],
        prints: &[i64],
        calls: [i64; 3],
    ) {
        self.reset();
        for (slot, word) in input.into_iter().enumerate() {
            self.store("%args", slot, word);
        }
        for (role, export) in bridge.callbacks.iter().enumerate() {
            // Actual prior returned state is the next input, with overlapping buffers.
            self.reset();
            self.expected.extend(prints);
            self.call(
                &export.symbol,
                "%args",
                if role == 0 { 8 } else { 9 },
                "%args",
                9,
                0,
            );
            for (slot, word) in state.iter().enumerate() {
                self.read("%args", slot, *word as i64);
            }
            for (index, name) in PROBES.into_iter().enumerate() {
                self.read(
                    &format!("@{name}_effect_probe"),
                    0,
                    if index < 3 {
                        calls[index]
                    } else {
                        i64::from(index - 3 == role)
                    },
                );
            }
            self.read("%storage", 0, CANARY);
            self.read("%storage", 10, CANARY);
        }
    }

    pub fn invalid(&mut self, bridge: &NativeSessionBridge, ty: &str) {
        for (role, export) in bridge.callbacks.iter().enumerate() {
            let count = if role == 0 { 8 } else { 9 };
            for (args, argc, out, slots) in [
                ("null", count, "%args", 9),
                ("%args", count - 1, "%args", 9),
                ("%args", count, "%args", 8),
                ("%args", count, "null", 9),
                ("%args", -1, "%args", 9),
            ] {
                self.reset();
                for slot in 0..9 {
                    self.store("%args", slot, CANARY as u64);
                }
                self.call(&export.symbol, args, argc, out, slots, 1);
                self.unchanged();
            }
            let mut bad = (0..4).map(|slot| (slot, 2)).collect::<Vec<_>>();
            match ty {
                "i32" => bad.extend([(4, u32::MAX as u64), (5, 1 << 32)]),
                "f32" => bad.extend([(4, 1 << 32), (5, u64::MAX)]),
                _ => {}
            }
            // Even an unused prior-result slot must pass preflight before effects.
            if role != 0 {
                match ty {
                    "i32" => bad.extend([(8, u32::MAX as u64), (8, 1 << 32)]),
                    "f32" => bad.extend([(8, 1 << 32), (8, u64::MAX)]),
                    _ => {}
                }
            }
            for (slot, word) in bad {
                self.reset();
                for slot in 0..9 {
                    self.store("%args", slot, 0);
                }
                self.store("%args", slot, word);
                for slot in 0..9 {
                    self.store("@effect_out", slot, CANARY as u64);
                }
                self.call(&export.symbol, "%args", count, "@effect_out", 9, 2);
                for slot in 0..9 {
                    self.read("@effect_out", slot, CANARY);
                }
                self.empty_probes();
            }
        }
    }

    fn unchanged(&mut self) {
        for slot in 0..9 {
            self.read("%args", slot, CANARY);
        }
        self.empty_probes();
        self.read("%storage", 0, CANARY);
        self.read("%storage", 10, CANARY);
    }

    fn empty_probes(&mut self) {
        for name in PROBES {
            self.read(&format!("@{name}_effect_probe"), 0, 0);
        }
    }

    pub fn trap(&mut self, bridge: &NativeSessionBridge, input: [u64; 8]) {
        self.reset();
        for (slot, word) in input.into_iter().enumerate() {
            self.store("%args", slot, word);
        }
        for slot in 0..9 {
            self.store("@effect_out", slot, CANARY as u64);
        }
        // Observe output before the original fatal trap, not a catch/retry path.
        self.llvm = self.llvm.replace(
            "  call void @llvm.trap()\n",
            "  call void @effect_trap_probe()\n",
        );
        self.call(&bridge.callbacks[0].symbol, "%args", 8, "@effect_out", 9, 0);
    }

    pub fn finish(mut self) -> (String, Vec<i64>) {
        self.llvm
            .push_str("  ret i64 0\n}\n\ndefine void @effect_trap_probe() {\n");
        for slot in 0..9 {
            self.llvm.push_str(&format!("  %p{slot} = getelementptr i64, ptr @effect_out, i64 {slot}\n  %v{slot} = load volatile i64, ptr %p{slot}\n  call void @nuis_debug_print_i64(i64 %v{slot})\n"));
        }
        self.llvm.push_str(
            "  %flushed = call i32 @fflush(ptr null)\n  call void @llvm.trap()\n  unreachable\n}\n",
        );
        (self.llvm, self.expected)
    }

    pub fn trap_canaries() -> Vec<i64> {
        vec![CANARY; 9]
    }
}
