pub fn source() -> String {
    let record = |initial: bool, shifted: bool| {
        let halves = ["Left", "Right"]
            .into_iter()
            .enumerate()
            .map(|(half, name)| {
                let parent = if half == 0 { "left" } else { "right" };
                let fields = (0..32)
                    .rev()
                    .map(|slot| {
                        let index = half * 32 + slot;
                        let value = if initial {
                            match index {
                                60 => "i32_from_i64(2147483646 + seed)".into(),
                                61 => "1.5".into(),
                                62 => "-1.5".into(),
                                63 => "seed > 0".into(),
                                _ => format!("seed + {index}"),
                            }
                        } else if shifted {
                            format!(
                                "carry.{parent}.f{slot}{}",
                                if index == 0 { " + i" } else { "" }
                            )
                        } else {
                            let field = format!("value.{parent}.f{slot}");
                            match index {
                                60 => format!("{field} + i32_from_i64(1)"),
                                61 | 62 => format!("{field} + 0.5"),
                                63 => format!("!{field}"),
                                _ => format!("{field} + 1"),
                            }
                        };
                        format!("f{slot}: {value}")
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{parent}: {name} {{ {fields} }}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("State {{ {halves} }}")
    };
    let definitions = ["Left", "Right"]
        .into_iter()
        .enumerate()
        .map(|(half, name)| {
            let fields = (0..32)
                .map(|slot| {
                    let ty = match half * 32 + slot {
                        60 => "i32",
                        61 => "f32",
                        62 => "f64",
                        63 => "bool",
                        _ => "i64",
                    };
                    format!("f{slot}: {ty}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("struct {name} {{ {fields} }}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "mod cpu Main {{
        {definitions}
        struct State {{ left: Left, right: Right }}
        @noinline fn relay(value: State) -> State {{ return {}; }}
        fn start(seed: i64) -> State {{ return {}; }}
        fn step(state: State) -> State {{
            let carry = state; let i = 0; let limit = state.left.f1;
            while i < limit {{
                let i = i + 1; let carry = relay(carry); let carry = {};
            }}
            return carry;
        }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}",
        record(false, false),
        record(true, false),
        record(false, true)
    )
}
