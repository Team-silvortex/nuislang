#[allow(dead_code)]
#[path = "unary_float_field_epoch_print_cases.rs"]
mod base;
pub use base::{Kind, KINDS};
pub const LOOP_LIMIT: u64 = 3;
pub const ENTRY_LIMIT: u64 = 64;

pub fn source(kind: Kind) -> String {
    include_str!("local_scalar_argument_print.ns").replace("f64", kind.ty())
}

// Independent bit-by-bit XOR and signed modulo arithmetic, not backend primitives.
pub fn xor_word(left: i64, right: i64) -> i64 {
    let modulus = 1i128 << 64;
    let (mut left, mut right) = (
        i128::from(left).rem_euclid(modulus),
        i128::from(right).rem_euclid(modulus),
    );
    let (mut bits, mut place) = (0i128, 1i128);
    for _ in 0..64 {
        if left % 2 != right % 2 {
            bits += place;
        }
        left /= 2;
        right /= 2;
        place *= 2;
    }
    signed(bits)
}

fn signed(value: i128) -> i64 {
    let bits = value.rem_euclid(1i128 << 64);
    (if bits >= 1i128 << 63 {
        bits - (1i128 << 64)
    } else {
        bits
    }) as i64
}

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub kind: Kind,
    pub initial: i64,
    pub selected: bool,
    pub trips: i64,
    pub left: i32,
    pub right: i32,
    pub mask: i64,
    pub guards: [i64; 4],
}

impl Case {
    pub fn words(self) -> Vec<i64> {
        let mut result = vec![
            self.initial,
            i64::from(self.selected),
            self.trips,
            i64::from(self.left),
            i64::from(self.right),
            self.mask,
        ];
        result.extend(self.guards);
        result
    }

    pub fn args(self) -> String {
        let mut result = vec![self.kind.decimal(self.initial), self.selected.to_string()];
        result.extend(self.words()[2..].iter().map(ToString::to_string));
        result.join(",")
    }

    pub fn execute(self, previous: Option<i64>, native: bool) -> Result<(i64, Vec<i64>), Vec<i64>> {
        let mut trace = vec![if previous.is_some() { 91 } else { 90 }, 70];
        let mut current = previous.unwrap_or(self.initial);
        let mut code = self.left;
        for _ in 0..self.trips {
            trace.push(71);
            let parameter = xor_word(i64::from(self.left) + 4294967296, self.mask);
            if !checked(parameter, self.guards[0], native, &mut trace) {
                return Err(trace);
            }
            trace.push(parameter);
            let pick = if self.selected {
                let value = xor_word(i64::from(code), parameter);
                if !checked(value, self.guards[1], native, &mut trace) {
                    return Err(trace);
                }
                value != 0
            } else {
                false
            };
            let bits = if pick {
                current
            } else {
                (current as u64 ^ self.kind.mask()) as i64
            };
            let value = if self.kind == Kind::F32 {
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
            current = (current as u64 ^ self.kind.mask()) as i64;
            code = self.right;
            let ready = xor_word(i64::from(code) + 4294967296, parameter);
            if !checked(ready, self.guards[2], native, &mut trace) {
                return Err(trace);
            }
            trace.extend([
                ready,
                signed(i128::from(ready) + if self.selected { 1 } else { -1 }),
            ]);
        }
        trace.push(73);
        if self.guards[3] == 0 {
            return Err(trace);
        }
        Ok((current, trace))
    }
}

fn checked(value: i64, guard: i64, native: bool, trace: &mut Vec<i64>) -> bool {
    if native {
        trace.extend([601, value, guard]);
    }
    guard != 0
}

pub const OBSERVERS: &[(&str, &str)] = &[("checked64",
    "  call void @word_print(i64 601)\n  call void @word_print(i64 %arg0)\n  call void @word_print(i64 %arg1)\n")];

pub fn seed(kind: Kind) -> Case {
    Case {
        kind,
        initial: kind.initial(),
        selected: true,
        trips: 1,
        left: i32::MIN,
        right: i32::MAX,
        mask: i64::MIN,
        guards: [1; 4],
    }
}

pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let words = if kind == Kind::F32 {
        vec![0, 0x80000000, kind.initial(), 1, 0x7f800000, kind.nan()]
    } else {
        vec![
            0,
            i64::MIN,
            kind.initial(),
            1,
            0x7ff0000000000000,
            kind.nan(),
        ]
    };
    let mut sequences = Vec::new();
    for word in words {
        for selected in [false, true] {
            let mut row = seed(kind);
            row.initial = word;
            row.selected = selected;
            if !selected {
                row.guards[1] = 0;
            }
            let mut next = row;
            next.initial = kind.nan(); // Events consume last State, not this decoy.
            next.trips = 2;
            next.left = -1;
            next.right = 0;
            next.mask = i64::MAX;
            let mut zero = next;
            zero.trips = 0;
            zero.guards = [0, 0, 0, 1];
            sequences.push(vec![row, next, zero, row]);
        }
    }
    for (selected, mask) in [(true, i64::MAX), (false, i64::MIN)] {
        let mut row = seed(kind);
        row.selected = selected;
        row.left = -7;
        row.right = row.left;
        row.mask = mask;
        // Equal codes cancel in the ready word; branch +/-1 crosses the signed boundary.
        sequences.push(vec![row, row]);
    }
    sequences
}

pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    sequences(kind)
        .into_iter()
        .filter(|rows| {
            let value = if kind == Kind::F32 {
                f64::from(f32::from_bits(rows[0].initial as u32))
            } else {
                f64::from_bits(rows[0].initial as u64)
            };
            value.is_finite()
        })
        .map(|mut rows| {
            for row in rows.iter_mut().skip(1) {
                row.initial = finite_decoy(kind);
            }
            rows
        })
        .collect()
}

fn finite_decoy(kind: Kind) -> i64 {
    if kind == Kind::F32 {
        0x411c0000
    } else {
        0x4023800000000000
    }
}

pub fn prefix(kind: Kind) -> Vec<Case> {
    let first = seed(kind);
    let mut second = first;
    second.initial = finite_decoy(kind);
    second.selected = false;
    second.guards[1] = 0;
    vec![first, second]
}

pub fn failures(kind: Kind) -> Vec<Case> {
    let mut result = Vec::new();
    for selected in [false, true] {
        for guard in [0, 2, 3] {
            let mut row = seed(kind);
            row.selected = selected;
            row.guards[guard] = 0;
            result.push(row);
        }
    }
    let mut lazy = seed(kind);
    lazy.guards[1] = 0;
    result.push(lazy);
    let mut empty = seed(kind);
    empty.trips = 0;
    empty.guards = [0; 4];
    result.push(empty);
    result
}
