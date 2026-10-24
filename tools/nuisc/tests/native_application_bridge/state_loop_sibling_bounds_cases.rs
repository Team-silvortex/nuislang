#[allow(dead_code)]
#[path = "state_loop_return_values_cases.rs"]
mod words;
pub use words::State;

#[path = "state_loop_return_projected_cases.rs"]
mod projected;

#[path = "state_loop_return_unprojected_cases.rs"]
mod unprojected;

#[path = "state_loop_return_conditional_cases.rs"]
mod conditional;

pub const LOOP_LIMIT: u64 = 18;
pub const ENTRY_LIMIT: u64 = 192;
pub const OBSERVER: &str =
    "  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n";

#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub leading: bool,
    pub mirrored: bool,
    pub preheader: bool,
    pub two_epochs: bool,
    pub snapshot_returns: bool,
    pub payload_calls: bool,
    pub field_payload: bool,
    pub mixed_payload: bool,
    pub projected_payload: bool,
    pub unprojected_payload: bool,
    pub conditional_payload: bool,
    pub scalar_join_payload: bool,
}

pub fn variants() -> Vec<Variant> {
    [false, true]
        .into_iter()
        .flat_map(|leading| {
            [false, true].map(move |mirrored| Variant {
                leading,
                mirrored,
                preheader: false,
                two_epochs: false,
                snapshot_returns: false,
                payload_calls: false,
                field_payload: false,
                mixed_payload: false,
                projected_payload: false,
                unprojected_payload: false,
                conditional_payload: false,
                scalar_join_payload: false,
            })
        })
        .collect()
}

pub fn preheader_variants() -> Vec<Variant> {
    variants()
        .into_iter()
        .map(|variant| Variant {
            preheader: true,
            ..variant
        })
        .collect()
}

pub fn epoch_variants() -> Vec<Variant> {
    preheader_variants()
        .into_iter()
        .map(|variant| Variant {
            two_epochs: true,
            ..variant
        })
        .collect()
}

pub fn snapshot_return_variants() -> Vec<Variant> {
    epoch_variants()
        .into_iter()
        .map(|variant| Variant {
            snapshot_returns: true,
            ..variant
        })
        .collect()
}

pub fn payload_call_variants() -> Vec<Variant> {
    snapshot_return_variants()
        .into_iter()
        .map(|variant| Variant {
            payload_calls: true,
            ..variant
        })
        .collect()
}

pub fn field_payload_variants() -> Vec<Variant> {
    payload_call_variants()
        .into_iter()
        .map(|variant| Variant {
            field_payload: true,
            ..variant
        })
        .collect()
}

pub fn mixed_payload_variants() -> Vec<Variant> {
    field_payload_variants()
        .into_iter()
        .map(|variant| Variant {
            mixed_payload: true,
            ..variant
        })
        .collect()
}

pub fn projected_payload_variants() -> Vec<Variant> {
    mixed_payload_variants()
        .into_iter()
        .map(|variant| Variant {
            projected_payload: true,
            ..variant
        })
        .collect()
}

pub fn payload_fault_modes(variant: Variant) -> std::ops::RangeInclusive<i64> {
    12..=if variant.unprojected_payload {
        35
    } else if variant.projected_payload {
        27
    } else {
        19
    }
}

pub fn unprojected_payload_variants() -> Vec<Variant> {
    projected_payload_variants()
        .into_iter()
        .map(|variant| Variant {
            unprojected_payload: true,
            ..variant
        })
        .collect()
}

pub fn source(variant: Variant) -> String {
    let mut source = include_str!("state_loop_sibling_bounds.ns").replace(
        "    @noinline fn work(",
        &format!(
            "{}\n    @noinline fn work(",
            include_str!("state_loop_return_shared_bound_helpers.ns")
        ),
    );
    if !variant.leading {
        for name in ["index", "sibling"] {
            let step = format!("{name} = {name} + 1;");
            assert_eq!(source.matches(&step).count(), 1);
            source = source.replace(&step, "");
            let mode = if name == "index" { 3 } else { 8 };
            source = source.replace(
                &format!("if mode == {mode} {{ continue; }}"),
                &format!("if mode == {mode} {{ {step} continue; }}"),
            );
            let print = if name == "index" {
                "print(944);"
            } else {
                "print(947);"
            };
            source = source.replace(print, &format!("{print} {step}"));
        }
    }
    if variant.mirrored {
        let begin = source.find("            if selected {").unwrap();
        let end = source
            .find("            } else { print(948); carry = advance(carry, 1); }")
            .unwrap();
        let body = source[begin + "            if selected {".len()..end].to_owned();
        source.replace_range(begin..end + "            } else { print(948); carry = advance(carry, 1); }".len(), &format!("            if selected != true {{ print(948); carry = advance(carry, 1); }} else {{{body}            }}"));
        for mode in 1..=9 {
            source = source.replace(
                &format!("if mode == {mode} {{"),
                &format!("if mode != {mode} {{}} else {{"),
            );
        }
    }
    if variant.preheader {
        let binding = "let frozen = bound_seed(carry, trips, root);";
        assert_eq!(source.matches(binding).count(), 1);
        source = source.replace(binding, "");
        let header = "        let mut outer = 0;";
        assert_eq!(source.matches(header).count(), 1);
        source = source.replace(
            header,
            &format!("        if selected {{\n        {binding}\n{header}"),
        );
        let tail = "        print(949);";
        assert_eq!(source.matches(tail).count(), 1);
        source = source.replace(tail, &format!(
            "        }} else {{\n            let mut inactive = 0;\n            while inactive < 2 {{\n                inactive = inactive + 1; print(941);\n                print(948); carry = advance(carry, 1);\n                if mode == 9 {{ return carry; }}\n            }}\n        }}\n{tail}"
        ));
    }
    if variant.two_epochs {
        assert!(variant.preheader);
        let bound = "let first_width = bound_extent(frozen, divisor, 1);";
        assert_eq!(source.matches(bound).count(), 1);
        source = source.replace(bound,
            "print(934);\n                let recent = bound_seed(carry, trips, root * (second + 3) + 0 / (second + 2));\n                let first_width = bound_extent(recent, divisor, 1);");
    }
    if variant.snapshot_returns {
        assert!(variant.two_epochs);
        for (early, late, record) in [(1, 10, "recent"), (6, 11, "frozen")] {
            let (old, new) = if variant.mirrored {
                (format!("if mode != {early} {{}} else {{ return carry; }}"),
                 format!("if mode != {early} && (mode != {late} || outer != 2) {{}} else {{ return {record}; }}"))
            } else {
                (format!("if mode == {early} {{ return carry; }}"),
                 format!("if mode == {early} || (mode == {late} && outer == 2) {{ return {record}; }}"))
            };
            assert_eq!(source.matches(&old).count(), 1);
            source = source.replace(&old, &new);
        }
    }
    if variant.payload_calls {
        let helpers = include_str!("state_loop_return_payload_helpers.ns");
        let helpers = if variant.field_payload {
            // Keep the native print policy limited to the selected call closure.
            helpers.split_once("@noinline fn payload_finish").unwrap().0
        } else {
            helpers
        };
        source = source.replace(
            "    @noinline fn work(",
            &format!("{helpers}\n    @noinline fn work("),
        );
        for (early, late, fault, delayed, record) in
            [(1, 10, 12, 18, "recent"), (6, 11, 15, 19, "frozen")]
        {
            let condition = format!("mode == {early} || (mode == {late} && outer == 2)");
            let inverse = format!("mode != {early} && (mode != {late} || outer != 2)");
            let binding = "let frozen = bound_seed(carry, trips, root);";
            assert_eq!(source.matches(binding).count(), 1);
            source = source.replace(binding, &format!(
                "let payload_{record}_early = mode == {early} || (mode >= {fault} && mode <= {});\n        let payload_{record}_late = mode == {late} || mode == {delayed};\n        {binding}", fault + 2
            ));
            let old = if variant.mirrored {
                inverse.clone()
            } else {
                condition
            };
            let new = if variant.mirrored {
                format!("payload_{record}_early != true && (payload_{record}_late != true || outer != 2)")
            } else {
                format!("payload_{record}_early || (payload_{record}_late && outer == 2)")
            };
            assert_eq!(source.matches(&old).count(), 1);
            source = source.replace(&old, &new);
            let old = format!("return {record};");
            assert_eq!(source.matches(&old).count(), 1);
            source = source.replace(&old, &format!(
                "return payload_finish({record}, payload_first(1 + 0 / (mode - {fault})), payload_second(2 + 0 / (mode - {})), (mode - {}) * (mode - {delayed}));", fault + 1, fault + 2
            ));
        }
    }
    if variant.field_payload {
        assert!(variant.payload_calls);
        source = source.replace(
            "    @noinline fn work(",
            &format!(
                "{}\n    @noinline fn work(",
                include_str!("state_loop_return_field_helpers.ns")
            ),
        );
        for record in ["recent", "frozen"] {
            let old = format!("return payload_finish({record},");
            assert_eq!(source.matches(&old).count(), 1);
            source = source.replace(&old, &format!(
                "return State {{ marker: field_marker({record}.marker), result: field_result({record}.result), gain: field_gain({record}.gain), total: field_total({record}.total,"
            ));
        }
        for delayed in [18, 19] {
            let old = format!("* (mode - {delayed}));");
            assert_eq!(source.matches(&old).count(), 1);
            source = source.replace(&old, &format!("* (mode - {delayed})) }};"));
        }
    }
    if variant.mixed_payload {
        assert!(variant.field_payload);
        source = source.replace(
            "    @noinline fn work(",
            &format!(
                "{}\n    @noinline fn work(",
                include_str!("state_loop_return_mixed_helpers.ns")
            ),
        );
        for (record, other) in [("recent", "frozen"), ("frozen", "recent")] {
            let old = format!("return State {{ marker: field_marker({record}.marker), result: field_result({record}.result), gain: field_gain({record}.gain), total: field_total({record}.total,");
            assert_eq!(source.matches(&old).count(), 1);
            source = source.replace(&old, &format!(
                "return State {{ marker: field_marker({other}.marker), result: field_result(carry.result != true), gain: field_gain({record}.gain), total: field_total(carry.total + frozen.total * 100 + recent.total * 10 + mixed_bias(carry.result),"
            ));
        }
    }
    if variant.projected_payload {
        assert!(variant.mixed_payload);
        source = projected::source(source);
    }
    if variant.unprojected_payload {
        assert!(variant.projected_payload);
        source = unprojected::source(source);
    }
    if variant.conditional_payload {
        assert!(variant.unprojected_payload);
        source = conditional::source(source);
    }
    if variant.scalar_join_payload {
        assert!(variant.conditional_payload);
        source = conditional::typed_source(source);
    }
    source
}

pub fn conditional_payload_variants() -> Vec<Variant> {
    unprojected_payload_variants()
        .into_iter()
        .map(|variant| Variant {
            conditional_payload: true,
            ..variant
        })
        .collect()
}

pub fn scalar_join_variants() -> Vec<Variant> {
    conditional_payload_variants()
        .into_iter()
        .map(|variant| Variant {
            scalar_join_payload: true,
            ..variant
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub struct Call {
    pub input: State,
    pub previous: Option<State>,
    pub selected: bool,
    pub trips: i64,
    pub divisor: i64,
    pub mode: i64,
    pub root: i64,
    pub second: i64,
    pub variant: Variant,
}

fn reserve(width: i64, budget: &mut Option<u64>) -> Option<()> {
    if let Some(left) = budget {
        let cost = width.max(0) as u64;
        *left = left.checked_sub(cost)?;
    }
    Some(())
}

fn advance(mut state: State, delta: i64, trace: &mut Vec<i64>) -> State {
    trace.push(960);
    state.total += delta;
    state.result = !state.result;
    state.gain ^= 1 << 31;
    state.marker ^= 1 << 63;
    state
}

fn snapshot(state: State, trips: i64, root: i64, trace: &mut Vec<i64>) -> Option<State> {
    trace.push(930);
    if root == 0 {
        return None;
    }
    let parity = state.total % 2;
    let frozen = trips / (1 + parity * parity);
    trace.push(931);
    Some(State {
        total: frozen,
        ..state
    })
}

fn payload(state: State, mode: i64, child: usize, trace: &mut Vec<i64>) -> Option<State> {
    let fault = if child == 0 { 12 } else { 15 };
    if mode == fault {
        return None;
    }
    trace.push(970);
    if mode == fault + 1 {
        return None;
    }
    trace.extend([971, 972]);
    if mode == fault + 2 || mode == if child == 0 { 18 } else { 19 } {
        return None;
    }
    trace.push(973);
    Some(State {
        total: state.total + 10,
        ..state
    })
}

impl Call {
    pub fn words(self) -> Vec<i64> {
        let mut words = self.input.words();
        words.extend([
            self.selected.into(),
            self.trips,
            self.divisor,
            self.mode,
            self.root,
            self.second,
        ]);
        words
    }
    pub fn args(self) -> String {
        self.words()
            .into_iter()
            .enumerate()
            .map(|(i, w)| match i {
                0 | 4 => (w != 0).to_string(),
                2 => f32::from_bits(w as u32).to_string(),
                3 => f64::from_bits(w as u64).to_string(),
                _ => w.to_string(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    pub fn execute(
        self,
        observed: bool,
        mut budget: Option<u64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        let mut trace = vec![if self.previous.is_some() { 951 } else { 950 }, 940];
        let result = (|| -> Option<State> {
            let mut state = self.previous.unwrap_or(self.input);
            let preheader = if self.variant.preheader && self.selected {
                Some(snapshot(state, self.trips, self.root, &mut trace)?)
            } else {
                None
            };
            reserve(2, &mut budget)?;
            for outer in 1..=2 {
                trace.push(941);
                if self.selected {
                    state = advance(state, outer, &mut trace);
                    let frozen = match preheader {
                        Some(frozen) => frozen,
                        None => snapshot(state, self.trips, self.root, &mut trace)?,
                    };
                    let recent = if self.variant.two_epochs {
                        trace.push(934);
                        let checked = 0i64.checked_div(self.second + 2)?;
                        snapshot(
                            state,
                            self.trips,
                            self.root * (self.second + 3) + checked,
                            &mut trace,
                        )?
                    } else {
                        frozen
                    };
                    for child in 0..2 {
                        if child == 1 {
                            if self.mode == 4 {
                                break;
                            }
                            if self.mode == 5 {
                                break;
                            }
                            if self.second == -1 {
                                return None;
                            }
                        }
                        trace.push(932);
                        if child == 1 && self.second == 0 {
                            return None;
                        }
                        trace.push(933);
                        let argument = if child == 0 { recent } else { frozen };
                        if observed {
                            trace.extend([argument.total, self.divisor]);
                        }
                        trace.push(16);
                        let width = argument.total.checked_div(self.divisor)?;
                        trace.push(if child == 0 { 942 } else { 945 });
                        reserve(width, &mut budget)?;
                        let mut index = 0;
                        while index < width {
                            if self.variant.leading {
                                index += 1;
                            }
                            trace.push(if child == 0 { 943 } else { 946 });
                            state =
                                advance(state, index + if child == 0 { 0 } else { 10 }, &mut trace);
                            if self.variant.payload_calls {
                                let fault = if child == 0 { 12 } else { 15 };
                                let returning = self.mode == if child == 0 { 1 } else { 6 }
                                    || (outer == 2
                                        && self.mode == if child == 0 { 10 } else { 11 })
                                    || (fault..=fault + 2).contains(&self.mode)
                                    || (outer == 2
                                        && self.mode == if child == 0 { 18 } else { 19 })
                                    || (self.variant.projected_payload
                                        && projected::returning(self.mode, child, outer))
                                    || (self.variant.unprojected_payload
                                        && unprojected::returning(self.mode, child, outer));
                                if returning {
                                    if self.variant.conditional_payload {
                                        return conditional::payload(
                                            state, frozen, recent, self.mode, child, &mut trace,
                                        );
                                    }
                                    if self.variant.unprojected_payload {
                                        return unprojected::payload(
                                            state, frozen, recent, self.mode, child, &mut trace,
                                        );
                                    }
                                    if self.variant.projected_payload {
                                        return projected::payload(
                                            state, frozen, recent, self.mode, child, &mut trace,
                                        );
                                    }
                                    if self.variant.field_payload {
                                        trace.extend([980, 981, 982]);
                                    }
                                    let argument = if self.variant.mixed_payload {
                                        trace.push(984);
                                        State {
                                            result: !state.result,
                                            total: state.total
                                                + frozen.total * 100
                                                + recent.total * 10
                                                + if state.result { 1000 } else { 2000 },
                                            gain: argument.gain,
                                            marker: if child == 0 {
                                                frozen.marker
                                            } else {
                                                recent.marker
                                            },
                                        }
                                    } else {
                                        argument
                                    };
                                    return payload(argument, self.mode, child, &mut trace);
                                }
                            }
                            if self.mode == if child == 0 { 1 } else { 6 } {
                                return Some(if self.variant.snapshot_returns {
                                    argument
                                } else {
                                    state
                                });
                            }
                            if self.variant.snapshot_returns
                                && outer == 2
                                && self.mode == if child == 0 { 10 } else { 11 }
                            {
                                return Some(argument);
                            }
                            if self.mode == if child == 0 { 2 } else { 7 } {
                                break;
                            }
                            if self.mode != if child == 0 { 3 } else { 8 } {
                                trace.push(if child == 0 { 944 } else { 947 });
                            }
                            if !self.variant.leading {
                                index += 1;
                            }
                        }
                    }
                    if self.mode == 4 {
                        break;
                    }
                    if self.mode == 5 {
                        continue;
                    }
                } else {
                    trace.push(948);
                    state = advance(state, 1, &mut trace);
                }
                if self.mode == 9 {
                    return Some(state);
                }
            }
            trace.push(949);
            Some(state)
        })();
        result.map(|state| (state, trace.clone())).ok_or(trace)
    }
    pub fn next(self) -> State {
        self.execute(false, None).unwrap().0
    }
}

pub fn base(variant: Variant) -> Call {
    Call {
        input: State {
            result: false,
            total: 0,
            gain: 0x80000000,
            marker: 0x8000000000000000,
        },
        previous: None,
        selected: true,
        trips: 4,
        divisor: 1,
        mode: 0,
        root: 1,
        second: 1,
        variant,
    }
}

pub fn sequence(mut calls: Vec<Call>) -> Vec<Call> {
    let mut previous = None;
    for call in &mut calls {
        call.previous = previous;
        previous = Some(call.next());
    }
    calls
}

pub fn prefix(variant: Variant, odd: bool) -> Vec<Call> {
    let mut call = base(variant);
    call.input.total = odd.into();
    call.selected = false;
    call.root = 0;
    call.divisor = 0;
    call.trips = i64::MAX;
    sequence(vec![call; 3])
}

pub fn failures(variant: Variant) -> Vec<(Vec<Call>, Call)> {
    let mut failures = (0..if variant.two_epochs { 7 } else { 5 })
        .map(|stage| {
            // The first division consumes the newer record in two-epoch variants.
            let overflow_odd = stage == 4 && (!variant.preheader || variant.two_epochs);
            let prefix = prefix(variant, overflow_odd);
            let mut call = base(variant);
            call.mode = 9;
            if variant.snapshot_returns {
                call.mode = if matches!(stage, 2 | 3) { 6 } else { 1 };
            }
            call.previous = Some(prefix.last().unwrap().next());
            match stage {
                0 => call.root = 0,
                1 => call.divisor = 0,
                2 => call.second = -1,
                3 => call.second = 0,
                4 => {
                    call.trips = i64::MIN;
                    call.divisor = -1;
                }
                5 => call.second = -2,
                6 => call.second = -3,
                _ => unreachable!(),
            }
            (prefix, call)
        })
        .collect::<Vec<_>>();
    if variant.payload_calls {
        for mode in payload_fault_modes(variant) {
            let prefix = prefix(variant, false);
            let mut call = base(variant);
            call.mode = mode;
            call.previous = Some(prefix.last().unwrap().next());
            if matches!(mode, 18 | 19 | 24..=27 | 32..=35) {
                call.trips = 5;
                call.divisor = 2;
            }
            failures.push((prefix, call));
        }
    }
    failures
}

pub fn healthy(variant: Variant) -> Vec<Vec<Call>> {
    let mut rows = vec![];
    for (gain, marker) in words::WORDS {
        for result in [false, true] {
            let modes: &[i64] = if variant.snapshot_returns {
                &[1, 6, 10, 11]
            } else {
                &[0, 1, 9]
            };
            for &mode in modes {
                let mut open = base(variant);
                open.input = State {
                    gain,
                    marker,
                    result,
                    ..open.input
                };
                open.selected = false;
                let mut event = base(variant);
                event.mode = mode;
                event.input.total = -999;
                event.input.gain = 9.25f32.to_bits();
                event.input.marker = 123.5f64.to_bits();
                if mode >= 10 {
                    event.trips = 5;
                    event.divisor = 2;
                }
                rows.push(sequence(vec![open, event, event]));
            }
        }
    }
    for mode in 0..=9 {
        let mut call = base(variant);
        call.mode = mode;
        if matches!(mode, 1 | 4 | 5) {
            call.second = 0;
        }
        rows.push(sequence(vec![call]));
    }
    if variant.snapshot_returns {
        for mode in [10, 11] {
            let mut call = base(variant);
            call.mode = mode;
            call.trips = 5;
            call.divisor = 2;
            rows.push(sequence(vec![call]));
        }
    }
    for trips in [0, -1, i64::MAX, i64::MIN] {
        let mut call = base(variant);
        call.trips = trips;
        call.selected = matches!(trips, 0 | -1);
        if !call.selected {
            call.root = 0;
            call.divisor = 0;
        }
        rows.push(sequence(vec![call]));
    }
    if variant.two_epochs {
        for second in [-3, -2] {
            let mut call = base(variant);
            call.selected = false;
            call.trips = i64::MAX;
            call.root = 0;
            call.second = second;
            rows.push(sequence(vec![call]));
        }
    }
    if variant.payload_calls {
        for mode in payload_fault_modes(variant) {
            let mut call = base(variant);
            call.mode = mode;
            call.selected = false;
            rows.push(sequence(vec![call]));
            call.selected = true;
            call.trips = 0;
            rows.push(sequence(vec![call]));
        }
    }
    rows
}
