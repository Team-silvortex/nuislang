use super::*;
use aggregate_loop_probe::{CallProbe, Invocation};

#[derive(Clone, Copy)]
struct Case {
    selected: bool,
    rounds: i64,
    width: i64,
    stride: i64,
    divisor: i64,
    seed: i64,
}

const BASE: Case = Case {
    selected: true,
    rounds: 3,
    width: 2,
    stride: 1,
    divisor: 3,
    seed: -17,
};

#[derive(Clone, Copy)]
enum Mode {
    Rectangular,
    Triangular,
    LateStride,
    Overwritten,
}

fn source(layers: usize, descending: bool, mode: Mode) -> String {
    let last = format!("child{}", layers - 1);
    let mut body = format!(
        "let flag = flag == false;
        let before_flag = flag;
        let cell = Cell {{ sum: cell.sum + word(flag), stamp: cell.stamp + 1 }};
        let before = cell;
        let cell = Cell {{ sum: cell.sum + {last}, stamp: cell.stamp + 1 }};
        let total = checked_value(total, divisor) + before.sum + {last} + word(before_flag);"
    );
    if matches!(mode, Mode::Overwritten) {
        body.push_str("let total = seed;");
    }
    for level in (0..layers).rev() {
        let parent = if level == 0 {
            "index".to_owned()
        } else {
            format!("child{}", level - 1)
        };
        let bound = if matches!(mode, Mode::Triangular) {
            parent.as_str()
        } else {
            "width"
        };
        let step = if matches!(mode, Mode::LateStride) {
            "stride + 2 - index"
        } else {
            "stride"
        };
        let (initial, compare, limit, update) = if descending {
            (bound, ">", "0", "-")
        } else {
            ("0", "<", bound, "+")
        };
        body = format!("let child{level}: i64 = {initial}; let step{level}: i64 = {step};
            while child{level} {compare} {limit} {{ let child{level} = child{level} {update} step{level}; {body} }}");
    }
    format!("mod cpu Main {{
        struct Cell {{ sum: i64, stamp: i64 }}
        struct State {{ value: i64, index: i64, sum: i64, stamp: i64, flag: i64, saved: i64, seed: i64 }}
        fn word(value: bool) -> i64 {{ if value {{ return 1; }} return 0; }}
        @noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value / divisor; }}
        @noinline fn leaf(value: i64, divisor: i64) -> i64 {{ return value; }}
        fn start(selected: bool, rounds: i64, width: i64, stride: i64, divisor: i64, seed: i64) -> State {{
            let index: i64 = 0;
            let total = seed;
            let flag = seed < 0;
            let saved = flag;
            let cell = Cell {{ sum: seed, stamp: 0 }};
            while index < rounds {{
                let index = index + 1;
                if selected {{ {body} }}
            }}
            return State {{ value: leaf(total, divisor), index: index, sum: cell.sum, stamp: cell.stamp,
                flag: word(flag), saved: word(saved), seed: seed }};
        }}
        fn step(state: State) -> State {{ return state; }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}")
}

enum Fault {
    Preflight,
    Arithmetic(&'static str),
}

fn induction(mut index: i64, limit: i64, stride: i64, descending: bool) -> Result<Vec<i64>, Fault> {
    let mut values = Vec::new();
    while if descending {
        index > limit
    } else {
        index < limit
    } {
        if stride <= 0 || values.len() == 65536 {
            return Err(Fault::Preflight);
        }
        index = if descending {
            index.checked_sub(stride)
        } else {
            index.checked_add(stride)
        }
        .ok_or(Fault::Preflight)?;
        values.push(index);
    }
    Ok(values)
}

struct Oracle {
    total: i64,
    sum: i64,
    stamp: i64,
    flag: bool,
    iterations: i64,
    calls: Vec<[i64; 3]>,
}

impl Oracle {
    fn child(
        &mut self,
        c: Case,
        layers: usize,
        parent: i64,
        outer: i64,
        descending: bool,
        mode: Mode,
    ) -> Result<(), Fault> {
        let bound = if matches!(mode, Mode::Triangular) {
            parent
        } else {
            c.width
        };
        let stride = if matches!(mode, Mode::LateStride) {
            c.stride.wrapping_add(2).wrapping_sub(outer)
        } else {
            c.stride
        };
        let (initial, limit) = if descending { (bound, 0) } else { (0, bound) };
        // Preflight this complete invocation, not the whole lexical loop tree.
        for index in induction(initial, limit, stride, descending)? {
            self.iterations += 1;
            if layers > 1 {
                self.child(c, layers - 1, index, outer, descending, mode)?;
            } else {
                self.flag = !self.flag;
                self.sum = self.sum.wrapping_add(i64::from(self.flag));
                let before = self.sum;
                self.sum = self.sum.wrapping_add(index);
                self.stamp = self.stamp.wrapping_add(2);
                self.calls.push([self.total, c.divisor, self.iterations]);
                if c.divisor == 0 {
                    return Err(Fault::Arithmetic("zero"));
                }
                if (self.total, c.divisor) == (i64::MIN, -1) {
                    return Err(Fault::Arithmetic("overflow"));
                }
                self.total = (i128::from(self.total) / i128::from(c.divisor)
                    + i128::from(before)
                    + i128::from(index)
                    + i128::from(self.flag)) as i64;
                if matches!(mode, Mode::Overwritten) {
                    self.total = c.seed;
                }
            }
        }
        Ok(())
    }
}

fn expected(c: Case, layers: usize, descending: bool, mode: Mode) -> (Invocation, Vec<[i64; 3]>) {
    let mut oracle = Oracle {
        total: c.seed,
        sum: c.seed,
        stamp: 0,
        flag: c.seed < 0,
        iterations: 0,
        calls: Vec::new(),
    };
    let mut outer = 0;
    let execution = (|| {
        for index in induction(0, c.rounds, 1, false)? {
            outer = index;
            oracle.iterations += 1;
            if c.selected {
                oracle.child(c, layers, index, index, descending, mode)?;
            }
        }
        Ok(())
    })();
    let success = execution.is_ok();
    let result = Invocation {
        arguments: vec![
            Value::Bool(c.selected),
            Value::Int(c.rounds),
            Value::Int(c.width),
            Value::Int(c.stride),
            Value::Int(c.divisor),
            Value::Int(c.seed),
        ],
        state: success.then(|| {
            vec![
                oracle.total,
                outer,
                oracle.sum,
                oracle.stamp,
                i64::from(oracle.flag),
                i64::from(c.seed < 0),
                c.seed,
            ]
        }),
        leaf: success.then_some([oracle.total, c.divisor]),
        iterations: oracle.iterations,
        reference_error: match execution {
            Err(Fault::Arithmetic(error)) => Some(error),
            _ => None,
        },
    };
    (result, oracle.calls)
}

fn execute(layers: usize, descending: bool, mode: Mode, cases: &[Case]) {
    let (observations, traces): (Vec<_>, Vec<_>) = cases
        .iter()
        .map(|&c| expected(c, layers, descending, mode))
        .unzip();
    aggregate_loop_probe::execute_probed(
        &source(layers, descending, mode),
        &observations,
        "loop_while_i64_body",
        true,
        None,
        Some(CallProbe {
            callee: "checked_value",
            traces: &traces,
        }),
    );
}

#[test]
fn literal_nested_loops_share_bool_flat_carries_and_independent_snapshots() {
    let mut cases = Vec::new();
    for rounds in [0, 1, 3] {
        for width in [0, 2, 3] {
            for seed in [i64::MIN, -17, 0, i64::MAX] {
                cases.push(Case {
                    rounds,
                    width,
                    seed,
                    ..BASE
                });
            }
        }
    }
    for descending in [false, true] {
        for (layers, mode) in [
            (1, Mode::Rectangular),
            (2, Mode::Triangular),
            (3, Mode::Rectangular),
        ] {
            execute(layers, descending, mode, &cases);
        }
    }
}

#[test]
fn literal_nested_loops_preflight_only_selected_invocations() {
    let skipped = [
        Case {
            rounds: 0,
            stride: 0,
            divisor: 0,
            ..BASE
        },
        Case {
            selected: false,
            width: 65537,
            stride: 0,
            divisor: 0,
            ..BASE
        },
        Case {
            width: 0,
            stride: 0,
            divisor: 0,
            ..BASE
        },
    ];
    for descending in [false, true] {
        for bad in [
            Case { stride: 0, ..BASE },
            Case { stride: -1, ..BASE },
            Case {
                width: 65537,
                ..BASE
            },
            Case {
                rounds: 65537,
                ..BASE
            },
        ] {
            let mut cases = skipped.to_vec();
            cases.push(bad);
            execute(1, descending, Mode::Rectangular, &cases);
        }
        execute(
            1,
            descending,
            Mode::LateStride,
            &[Case { rounds: 2, ..BASE }, BASE],
        );
    }
    // The first child step fits, but the complete child induction would wrap.
    execute(
        1,
        false,
        Mode::Rectangular,
        &[Case {
            width: i64::MAX,
            stride: i64::MAX - 1,
            ..BASE
        }],
    );
}

#[test]
fn literal_nested_loops_keep_selected_and_overwritten_arithmetic_failures() {
    for descending in [false, true] {
        for mode in [Mode::Rectangular, Mode::Overwritten] {
            for bad in [
                Case { divisor: 0, ..BASE },
                Case {
                    seed: i64::MIN,
                    divisor: -1,
                    ..BASE
                },
            ] {
                execute(1, descending, mode, &[BASE, bad]);
            }
        }
    }
}
