pub const CASES: &[&str] = &[
    "correlated",
    "nested",
    "separate-calls",
    "old-version",
    "selected-check",
];

pub fn source(case: &str, choose: bool, limit: i64) -> String {
    let arm = |input: &str| {
        format!(
            "let snapshot = relay({input}); let carry = snapshot; let saved = {};",
            if case == "separate-calls" {
                "relay(snapshot)"
            } else {
                "snapshot"
            },
        )
    };
    let then_body = if case == "nested" {
        format!(
            "if choose {{ {} }} else {{ {} }}",
            arm("seed"),
            arm("other")
        )
    } else {
        format!(
            "{} {}",
            arm("seed"),
            if case == "selected-check" {
                "let selected_check = 10 / limit;"
            } else {
                ""
            }
        )
    };
    format!("mod cpu Main {{
        struct State {{ value: i64, tag: i64 }}
        @noinline fn relay(value: State) -> State {{
            return State {{ value: value.value, tag: value.tag + 1 }};
        }}
        fn step(seed: State, other: State, choose: bool, limit: i64) -> State {{
            let carry = seed; let saved = carry; let i = 0;
            if choose {{ {then_body} }} else {{ {} }}
            {}
            while i < limit {{
                let i = i + 1; let carry = carry; let old = carry;
                let carry = State {{ value: old.value + 1, tag: saved.tag }};
                if i == 2 {{ return carry; }}
            }}
            return carry;
        }}
        fn main() -> i64 {{
            let result = step(State {{ value: 10, tag: 7 }}, State {{ value: 40, tag: 17 }}, {choose}, {limit});
            return result.value + result.tag;
        }}
    }}", arm("other"), if case == "old-version" { "let carry = relay(carry);" } else { "" })
}

pub fn expected(case: &str, choose: bool, limit: i64) -> Option<i64> {
    if case == "selected-check" && choose && limit == 0 {
        return None;
    }
    let mut result = if choose { 18 } else { 58 } + limit.min(2);
    if (case == "separate-calls" && limit > 0) || (case == "old-version" && limit == 0) {
        result += 1;
    }
    Some(result)
}
