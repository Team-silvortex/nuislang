pub(super) const SOURCE: &str = include_str!("return_exit_prefix.ns");

#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub seed: i64,
    pub pick: bool,
    pub take: bool,
    pub inner: bool,
}

impl Case {
    pub fn words(self) -> Vec<i64> {
        vec![
            self.seed,
            self.pick.into(),
            self.take.into(),
            self.inner.into(),
            1,
            1,
            self.take.into(),
            self.take.into(),
            (self.take && self.inner).into(),
            1,
            (!self.take).into(),
        ]
    }

    pub fn next(self) -> Self {
        Self {
            take: !self.take,
            inner: !self.inner,
            ..self
        }
    }

    pub fn fields(self) -> [i64; 3] {
        let mut a = -1 - self.seed;
        let mut d = true;
        let local = self.seed + if self.take { 2 } else { 1 };
        if self.take && self.inner {
            a = -1 - a;
            d = !d;
        }
        [local, a, d.into()]
    }

    pub fn result(self) -> i64 {
        let [local, a, d] = self.fields();
        (a < 0 && d != 0 && local == 9).into()
    }

    pub fn trace(self, mirrored: bool, observe: bool) -> Vec<i64> {
        let mut trace = vec![10, 11];
        trace.extend(if self.pick != mirrored {
            [12, 13]
        } else {
            [13, 12]
        });
        trace.push(14);
        if self.take {
            trace.extend([20, 21]);
            if self.inner {
                trace.extend([22, 23]);
            }
        } else {
            trace.extend([25, 26]);
        }
        if observe {
            trace.extend(self.fields());
        }
        trace.push(28);
        trace
    }
}

pub(super) fn cases() -> Vec<Case> {
    let mut result = Vec::new();
    for seed in [-2, 7, 8] {
        for pick in [false, true] {
            for take in [false, true] {
                for inner in [false, true] {
                    result.push(Case {
                        seed,
                        pick,
                        take,
                        inner,
                    });
                }
            }
        }
    }
    result
}

// Each row poisons one distinct evaluated argument boundary. The shared child
// operand first traps at 22, and the shared tail operand first traps at 25.
pub(super) fn failures() -> Vec<(Case, usize, i64)> {
    [
        (4, 11, true),
        (5, 14, true),
        (6, 20, true),
        (7, 21, true),
        (8, 22, true),
        (9, 28, true),
        (10, 25, false),
    ]
    .into_iter()
    .map(|(slot, stage, take)| {
        (
            Case {
                seed: 7,
                pick: slot % 2 == 0,
                take,
                inner: true,
            },
            slot,
            stage,
        )
    })
    .collect()
}
