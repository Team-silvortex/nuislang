#[allow(dead_code)]
#[path = "local_scalar_argument_print_cases.rs"]
mod base;
pub use base::{Kind, KINDS};
pub const LOOP_LIMIT: u64 = 3;
pub const ENTRY_LIMIT: u64 = 64;
pub const OBSERVERS: &[(&str, &str)] = base::OBSERVERS;

pub fn source(kind: Kind) -> String {
    include_str!("epoch_local_argument_print.ns").replace("f64", kind.ty())
}

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub base: base::Case,
}

impl Case {
    pub fn kind(self) -> Kind {
        self.base.kind
    }
    pub fn words(self) -> Vec<i64> {
        self.base.words()
    }
    pub fn args(self) -> String {
        self.base.args()
    }
    pub fn execute(self, previous: Option<i64>, native: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let row = self.base;
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        let mut current = previous.unwrap_or(row.initial);
        let mut code = row.left;
        for _ in 0..row.trips {
            trace.push(71);
            // Reuse the independent bit-by-bit word oracle, not backend XOR.
            let word = base::xor_word(i64::from(code) + 4294967296, row.mask);
            if !checked(word, row.guards[0], native, &mut trace) {
                return Err(trace);
            }
            let pick = if row.selected {
                if !checked(word, row.guards[1], native, &mut trace) {
                    return Err(trace);
                }
                word != 0
            } else {
                false
            };
            let bits = if pick {
                current
            } else {
                (current as u64 ^ row.kind.mask()) as i64
            };
            let value = if row.kind == Kind::F32 {
                f64::from(f32::from_bits(bits as u32))
            } else {
                f64::from_bits(bits as u64)
            };
            trace.push(if value < 0.0 {
                -1
            } else if value == 0.0 {
                0
            } else if value.is_nan() {
                2
            } else {
                1
            });
            trace.push(74);
            if row.guards[2] == 0 {
                return Err(trace);
            }
            current = (current as u64 ^ row.kind.mask()) as i64;
            code = row.right;
            trace.push(75);
        }
        trace.push(73);
        if row.guards[3] == 0 {
            return Err(trace);
        }
        Ok((current, trace))
    }
}

fn checked(word: i64, guard: i64, native: bool, trace: &mut Vec<i64>) -> bool {
    if native {
        trace.extend([601, word, guard]);
    }
    guard != 0
}

pub fn seed(kind: Kind) -> Case {
    Case {
        base: base::seed(kind),
    }
}
fn zero_word(kind: Kind, selected: bool) -> Case {
    let mut case = seed(kind);
    case.base.selected = selected;
    case.base.mask = i64::from(case.base.left) + 4294967296;
    if !selected {
        case.base.guards[1] = 0;
    }
    case
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = base::sequences(kind)
        .into_iter()
        .map(|rows| rows.into_iter().map(|base| Case { base }).collect())
        .collect::<Vec<_>>();
    for selected in [false, true] {
        rows.push(vec![zero_word(kind, selected)]);
    }
    rows
}
pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = base::cli_sequences(kind)
        .into_iter()
        .map(|rows| rows.into_iter().map(|base| Case { base }).collect())
        .collect::<Vec<_>>();
    for selected in [false, true] {
        rows.push(vec![zero_word(kind, selected)]);
    }
    rows
}
pub fn prefix(kind: Kind) -> Vec<Case> {
    base::prefix(kind)
        .into_iter()
        .map(|base| Case { base })
        .collect()
}
pub fn failures(kind: Kind) -> Vec<Case> {
    let mut rows = base::failures(kind)
        .into_iter()
        .map(|base| Case { base })
        .collect::<Vec<_>>();
    let mut zero = zero_word(kind, true);
    zero.base.guards[1] = 0;
    rows.push(zero);
    rows
}
