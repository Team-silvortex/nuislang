use super::NativeSessionBridge;

struct Driver {
    body: String,
    next: usize,
    output: Vec<u64>,
}
impl Driver {
    fn store(&mut self, base: &str, index: usize, word: u64) {
        let id = self.next;
        self.next += 1;
        self.body.push_str(&format!("  %p{id} = getelementptr i64, ptr %{base}, i64 {index}\n  store i64 {}, ptr %p{id}, align 8\n", word as i64));
    }
    fn call(
        &mut self,
        symbol: &str,
        args: &str,
        count: i64,
        out: &str,
        slots: i64,
        status: u64,
        words: &[u64],
    ) {
        let id = self.next;
        self.next += 1;
        self.body.push_str(&format!("  %status{id} = call i32 @{symbol}(ptr {args}, i64 {count}, ptr {out}, i64 {slots})\n  %wide{id} = sext i32 %status{id} to i64\n  call void @nuis_debug_print_i64(i64 %wide{id})\n"));
        self.output.push(status);
        for (index, expected) in words.iter().enumerate() {
            let id = self.next;
            self.next += 1;
            self.body.push_str(&format!("  %p{id} = getelementptr i64, ptr %state, i64 {index}\n  %v{id} = load i64, ptr %p{id}, align 8\n  call void @nuis_debug_print_i64(i64 %v{id})\n"));
            self.output.push(*expected);
        }
    }
}

pub(super) fn build(
    bridge: &NativeSessionBridge,
    gain: u32,
    scale: u64,
    states: &[Vec<u64>],
) -> (String, Vec<u64>) {
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    llvm.push_str("\n@bridge_entries = internal global i64 0, align 8\n");
    for export in &bridge.callbacks {
        let start = llvm
            .find(&format!("define i64 @nuis_fn_{}(", export.function))
            .unwrap();
        let insertion = start + llvm[start..].find("{\n").unwrap() + 2;
        llvm.insert_str(insertion, "  %bridge_entry_count = load i64, ptr @bridge_entries, align 8\n  %bridge_entry_next = add i64 %bridge_entry_count, 1\n  store i64 %bridge_entry_next, ptr @bridge_entries, align 8\n");
    }
    let symbols = bridge
        .callbacks
        .iter()
        .map(|export| export.symbol.as_str())
        .collect::<Vec<_>>();
    let mut driver = Driver { body: "\ndefine i64 @nuis_yir_entry() {\n  %state = alloca [8 x i64], align 8\n  %bad = alloca [8 x i64], align 8\n".to_owned(), next: 0, output: Vec::new() };
    for (index, value) in [10, -17_i64 as u64, 1, gain as u64, scale]
        .into_iter()
        .enumerate()
    {
        driver.store("state", index, value);
    }
    driver.call(symbols[0], "%state", 5, "%state", 6, 0, &states[0]);
    for (i, (delta, paused)) in [(3_i64, false), (100, true), (-2, false)]
        .into_iter()
        .enumerate()
    {
        driver.store("state", 6, delta as u64);
        driver.store("state", 7, u64::from(paused));
        driver.call(symbols[1], "%state", 8, "%state", 6, 0, &states[i + 1]);
    }
    driver.store("state", 6, 5);
    driver.call(symbols[2], "%state", 7, "%state", 6, 0, &states[4]);
    // Invalid shapes never dereference null inputs, enter a callback or overwrite state.
    for (args, count, out, slots) in [
        ("null", 5, "%state", 6),
        ("%bad", 4, "%state", 6),
        ("%bad", 5, "%state", 5),
        ("%bad", 5, "null", 6),
        ("%bad", -1, "%state", 6),
    ] {
        driver.call(symbols[0], args, count, out, slots, 1, &states[4]);
    }
    for (bad_index, bad_word) in [(2, 2), (1, u32::MAX as u64), (3, 1_u64 << 32)] {
        for (index, value) in [10, -17_i64 as u64, 1, gain as u64, scale]
            .into_iter()
            .enumerate()
        {
            driver.store(
                "bad",
                index,
                if index == bad_index { bad_word } else { value },
            );
        }
        driver.call(symbols[0], "%bad", 5, "%state", 6, 2, &states[4]);
    }
    driver.body.push_str("  %entered = load i64, ptr @bridge_entries, align 8\n  call void @nuis_debug_print_i64(i64 %entered)\n  ret i64 0\n}\n");
    driver.output.push(5);
    llvm.push_str(&driver.body);
    (llvm, driver.output)
}
