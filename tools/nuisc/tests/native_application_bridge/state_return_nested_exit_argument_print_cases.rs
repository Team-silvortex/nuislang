#[allow(dead_code)]
#[path = "unary_float_field_epoch_print_cases.rs"]
mod floats;
pub use floats::{Kind, KINDS};
pub type State = [i64; 2];
pub const SOURCE: &str = include_str!("state_return_nested_exit_argument_print.ns");
pub const LOOP_LIMIT: u64 = 12;
pub const ENTRY_LIMIT: u64 = 192;
pub const LEFT_TAG: i64 = 34359738368;

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub kind: Kind,
    pub initial: i64,
    pub control: i64,
    pub outer: bool,
    pub inner: bool,
    pub trips: i64,
    pub faults: [i64; 5],
}
impl Case {
    pub fn kind(self) -> Kind {
        self.kind
    }
    pub fn words(self) -> Vec<i64> {
        let mut words = vec![
            self.initial,
            self.control,
            i64::from(self.outer),
            i64::from(self.inner),
            self.trips,
        ];
        words.extend(self.faults);
        words
    }
    pub fn args(self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.kind.decimal(self.initial),
            self.control,
            self.outer,
            self.inner,
            self.trips,
            self.faults
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
    pub fn execute(
        self,
        prior: Option<State>,
        native: bool,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        self.execute_tagged(prior, native, LEFT_TAG)
    }
    pub fn execute_tagged(
        self,
        prior: Option<State>,
        _native: bool,
        tag: i64,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        let mut state = prior.unwrap_or([self.initial, self.control]);
        let mut trace = vec![if prior.is_some() { 91 } else { 90 }, 70];
        for index in 0..self.trips.max(0) {
            trace.push(71);
            let guards = self.faults.map(|v| v.wrapping_sub(index));
            let joined = if self.outer {
                let input = if self.inner {
                    state[1].wrapping_add(1)
                } else {
                    state[1].wrapping_sub(1)
                };
                trace.extend([80, input]);
                if guards[usize::from(!self.inner)] == 0 {
                    return Err(trace);
                }
                let local = input ^ 4294967296;
                trace.extend([81, if self.inner { 101 } else { 103 }, local]);
                let staged = local ^ if self.inner { tag } else { 68719476736 };
                trace.extend([97, staged]);
                if guards[2] == 0 {
                    return Err(trace);
                }
                trace.push(98);
                staged ^ 17179869184
            } else {
                let staged = state[1] ^ 137438953472;
                trace.extend([99, staged]);
                if guards[3] == 0 {
                    return Err(trace);
                }
                trace.push(100);
                staged
            };
            trace.extend([95, joined]);
            if guards[4] == 0 {
                return Err(trace);
            }
            trace.push(96);
            state = [(state[0] as u64 ^ self.kind.mask()) as i64, joined];
            trace.push(72);
        }
        trace.push(73);
        Ok((state, trace))
    }
}
pub fn seed(kind: Kind) -> Case {
    Case {
        kind,
        initial: kind.initial(),
        control: i64::MAX,
        outer: true,
        inner: true,
        trips: 3,
        faults: [16; 5],
    }
}
pub fn prefix(kind: Kind) -> Vec<Case> {
    let first = seed(kind);
    let mut second = seed(kind);
    second.inner = false;
    second.initial = 0;
    second.control = 7;
    vec![first, second]
}
pub fn poisons(kind: Kind) -> Vec<Vec<Case>> {
    (0..3)
        .map(|path| {
            let mut row = seed(kind);
            row.outer = path != 2;
            row.inner = path == 0;
            row.faults = match path {
                0 => [16, 0, 16, 0, 16],
                1 => [0, 16, 16, 0, 16],
                _ => [0, 0, 0, 16, 16],
            };
            vec![row, seed(kind)]
        })
        .collect()
}
pub fn sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = poisons(kind);
    for control in [i64::MIN, i64::MAX, -1, 0, 4294967296] {
        for (outer, inner) in [(true, true), (true, false), (false, true), (false, false)] {
            let mut first = seed(kind);
            first.control = control;
            first.outer = outer;
            first.inner = inner;
            let mut zero = first;
            zero.trips = 0;
            zero.initial = 0;
            zero.control = 13;
            let mut changed = first;
            changed.outer = !outer;
            changed.inner = !inner;
            rows.push(vec![first, zero, changed, first]);
        }
    }
    for word in [
        0,
        kind.mask() as i64,
        kind.nan(),
        (kind.nan() as u64 ^ kind.mask()) as i64,
    ] {
        let mut first = seed(kind);
        first.initial = word;
        rows.push(vec![first, first]);
    }
    rows
}
pub fn failure_sequences(kind: Kind) -> Vec<(Vec<Case>, Case)> {
    let mut rows = Vec::new();
    for which in 0..5 {
        for at in 0..3 {
            let mut bad = seed(kind);
            bad.inner = which != 1;
            bad.outer = which != 3;
            bad.faults[which] = at;
            rows.push((prefix(kind), bad));
        }
    }
    for which in [0, 1, 2, 3] {
        let mut bad = seed(kind);
        bad.inner = which != 1;
        bad.outer = which != 3;
        bad.faults[which] = 1;
        bad.faults[4] = 1;
        if which < 2 {
            bad.faults[2] = 1;
        }
        rows.push((prefix(kind), bad));
    }
    rows
}
pub fn cli_sequences(kind: Kind) -> Vec<Vec<Case>> {
    let mut rows = poisons(kind);
    rows.push(prefix(kind));
    rows
}
pub fn delivery_failure_sequences(kind: Kind) -> Vec<(Vec<Case>, Case)> {
    let all = failure_sequences(kind);
    [0, 3, 6, 9, 12, 2, 5, 17]
        .into_iter()
        .map(|i| all[i].clone())
        .collect()
}
