pub const CASES: [(&str, usize, usize); 6] = [
    ("direct", 1, 1),
    ("ordinary_break", 1, 0),
    ("child_break", 2, 1),
    ("parent_break", 2, 1),
    ("continue", 1, 1),
    ("source_spoof", 1, 0),
];

pub fn source(case: &str, leading: bool, limit: usize, stop: usize) -> String {
    let effects = match case {
        "direct" => "if index == stop { return result + index * 100; } let result = result + 1;",
        "ordinary_break" => {
            "if index == 2 { break; }
            if index == stop { return result + index * 100; } let result = result + 1;"
        }
        "child_break" => {
            "let child = 0; while child < index {
                let child = child + 1; if child == 1 { break; }
            }
            if index == stop { return result + child + index * 100; }
            let result = result + child + 1;"
        }
        "parent_break" => {
            "if index == 2 { break; }
            let child = 0; while child < index {
                let child = child + 1;
                if child == stop { return result + child + index * 100; }
            }
            let result = result + child + 1;"
        }
        "continue" => {
            "if index == 1 { CONTINUE_STEP continue; }
            if index == stop { return result + index * 100; } let result = result + 1;"
        }
        "source_spoof" => {
            "if index == 2 { let __nuis_return_pending_0: i64 = 1; break; }
            if index == stop { return result + index * 100; } let result = result + 1;"
        }
        _ => panic!("unknown case"),
    }
    .replace(
        "CONTINUE_STEP",
        if leading {
            ""
        } else {
            "let index = index + 1;"
        },
    );
    let step = "let index = index + 1;";
    format!(
        "mod cpu Main {{
            fn work(limit: i64, stop: i64) -> i64 {{
                let __nuis_return_pending_0 = 0;
                let result = 10; let index = 0;
                if limit >= 0 {{
                    while index < limit {{
                        {} let result = result; {effects} {}
                    }}
                }}
                return result + index * 1000 + __nuis_return_pending_0 * 10000;
            }}
            fn main() -> i64 {{ return work({limit}, {stop}) % 251; }}
        }}",
        if leading { step } else { "" },
        if leading { "" } else { step },
    )
}

pub fn expected(case: &str, leading: bool, limit: usize, stop: usize) -> i64 {
    let mut result = 10;
    let mut index = 0;
    let mut spoof = 0;
    while index < limit {
        if leading {
            index += 1;
        }
        if matches!(case, "ordinary_break" | "parent_break" | "source_spoof") && index == 2 {
            spoof = usize::from(case == "source_spoof");
            break;
        }
        if case == "continue" && index == 1 {
            if !leading {
                index += 1;
            }
            continue;
        }
        let child = match case {
            "child_break" => usize::from(index > 0),
            "parent_break" => {
                if stop > 0 && stop <= index {
                    return ((result + stop + index * 100) % 251) as i64;
                }
                index
            }
            _ => 0,
        };
        if case != "parent_break" && index == stop {
            return ((result + child + index * 100) % 251) as i64;
        }
        result += child + 1;
        if !leading {
            index += 1;
        }
    }
    ((result + index * 1000 + spoof * 10000) % 251) as i64
}
