pub(super) const TYPES: [&str; 3] = ["i32", "f32", "f64"];
pub(super) const MODES: [&str; 5] = [
    "complete",
    "partial",
    "exit-only",
    "complete-no-exit",
    "partial-no-exit",
];

pub(super) fn source(ty: &str, mode: &str, constant: bool) -> String {
    let binding = if constant { "const" } else { "let" };
    let finish = if ty == "i32" { "value" } else { "-value" };
    let tail = match mode.strip_suffix("-no-exit").unwrap_or(mode) {
        "complete" => "return finish(selected);",
        "partial" => "let saved = finish(selected); if nested { return saved; }",
        "exit-only" => "let saved = finish(selected);",
        _ => unreachable!(),
    };
    let params = format!(
        "outer: bool, gate: bool, nested: bool, early: bool,
         value: {ty}, fallback: {ty}, left: i64, right: i64"
    );
    let args = "outer, gate, nested, early, value, fallback, left, right";
    let state_args = "state.outer, state.gate, state.nested, state.early,
        state.value, state.fallback, state.left, state.right";
    let fields = "outer: outer, gate: gate, nested: nested, early: early,
        value: value, fallback: fallback, left: left, right: right";
    let state_fields = "outer: state.outer, gate: state.gate, nested: state.nested,
        early: state.early, value: state.value, fallback: state.fallback,
        left: state.left, right: state.right";
    let text = format!(
        "mod cpu Main {{
        @noinline fn choose(value: bool) -> bool {{ return value; }}
        @noinline fn observe(value: {ty}, divisor: i64) -> {ty} {{
            let unused = 10 / divisor; return value;
        }}
        @noinline fn finish(value: {ty}) -> {ty} {{ return {finish}; }}
        @noinline fn event({params}) -> {ty} {{
            print(99);
            if outer {{
                {binding} selected: {ty} = if choose(gate) {{
                    if early {{ print(72); return fallback; }}
                    let local: {ty} = observe(value, left); print(70); local
                }} else {{
                    if early {{ print(72); return fallback; }}
                    let local: {ty} = observe(fallback, right); print(71); local
                }};
                print(80);
                {tail}
            }}
            print(77); return fallback;
        }}
        struct State {{ {params}, result: {ty} }}
        fn start({params}) -> State {{
            return State {{ {fields}, result: event({args}) }};
        }}
        fn step(state: State) -> State {{
            return State {{ {state_fields}, result: event({state_args}) }};
        }}
        fn stop(state: State) -> State {{
            return State {{ {state_fields}, result: event({state_args}) }};
        }}
        fn main() -> i64 {{ return 0; }}
    }}"
    );
    if mode.ends_with("-no-exit") {
        text.replace("if early { print(72); return fallback; }", "")
    } else {
        text
    }
}
