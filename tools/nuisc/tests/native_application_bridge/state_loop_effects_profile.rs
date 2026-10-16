use super::cases;

pub const WORK_OBSERVER: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n  call void @nuis_debug_print_i64(i64 %arg2)\n";

pub fn source_variants() -> [String; 4] {
    cases::source_variants().map(|source| {
        source
            .replace(
                "{ return value == (proof == 1); }",
                "{ print(10); return value == (proof == 1); }",
            )
            .replace(
                "{ return value * 2 + proof + index; }",
                "{ print(20); return value * 2 + proof + index; }",
            )
            .replace(
                "            let local = refine(",
                "            print(19);\n            let local = refine(",
            )
    })
}

pub fn trace(call: cases::Invocation, observed: bool) -> Vec<i64> {
    let mut trace = vec![5];
    trace.extend(iteration_trace(&call.iterations().unwrap(), observed));
    trace.push(35);
    if observed {
        trace.extend([call.next().total, call.prior().into(), 1]);
    }
    trace.push(40);
    trace
}

fn iteration_trace(rows: &[cases::Iteration], observed: bool) -> Vec<i64> {
    let mut trace = vec![];
    for row in rows {
        trace.push(10);
        if row.selected {
            trace.push(19);
            if observed {
                trace.extend([row.entering, row.proof, row.index]);
            }
            trace.push(20);
        }
    }
    trace
}

pub fn before_failure(call: cases::Invocation, bad: Option<usize>) -> Vec<i64> {
    if bad == Some(9) {
        return vec![];
    }
    let mut trace = vec![5];
    if bad == Some(4) {
        return trace;
    }
    match call.iterations() {
        Err(rows) => {
            trace.extend(iteration_trace(&rows, true));
            if call.root != 0 {
                trace.extend([10, 19]);
            }
        }
        Ok(rows) => {
            trace.extend(iteration_trace(&rows, true));
            if bad == Some(7) {
                trace.push(35);
            } else {
                assert_eq!(bad, Some(6));
            }
        }
    }
    trace
}
