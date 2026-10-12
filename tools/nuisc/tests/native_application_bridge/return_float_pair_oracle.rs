pub struct Outcome {
    pub result: bool,
    pub trace: Vec<i32>,
    pub child: Option<u64>,
}

// Independent IEEE comparisons and sign-bit updates, not a compiler codec.
pub fn oracle(ty: &str, word: u64, first: bool, second: bool) -> Outcome {
    let sign = if ty == "f32" { 1 << 31 } else { 1 << 63 };
    let negative = |bits| {
        if ty == "f32" {
            f32::from_bits(bits as u32) < 0.0
        } else {
            f64::from_bits(bits) < 0.0
        }
    };
    let equal = |bits| {
        if ty == "f32" {
            f32::from_bits(bits as u32) == f32::from_bits(word as u32)
        } else {
            f64::from_bits(bits) == f64::from_bits(word)
        }
    };
    let mut trace = vec![99, 90, 87];
    let mut local = negative(word);
    let mut observed = None;
    let result;
    if first {
        if second {
            trace.extend([91, 92]);
            let mut child = word;
            if local {
                child ^= sign;
                trace.extend([93, 95]);
                if negative(child) {
                    local = true;
                    trace.push(93);
                } else {
                    local = false;
                }
            } else {
                local = equal(child);
                trace.extend([95, 93]);
                if local {
                    child ^= sign;
                    trace.push(93);
                    local = !equal(child);
                }
                local = local && first;
            }
            if local {
                child ^= sign;
                trace.push(93);
            }
            observed = Some(child);
            trace.push(96);
            local = equal(child);
            trace.push(83);
        }
        result = if local {
            trace.push(97);
            first
        } else {
            false
        };
    } else {
        result = if local {
            true
        } else {
            trace.push(97);
            second
        };
    }
    trace.push(79);
    Outcome {
        result,
        trace,
        child: observed,
    }
}

pub const STAGES: [i32; 9] = [90, 87, 91, 92, 93, 95, 96, 83, 97];

pub fn arguments(ty: &str, word: u64, first: bool, second: bool) -> Vec<u64> {
    let trace = oracle(ty, word, first, second).trace;
    let mut values = vec![u64::from(first), u64::from(second)];
    values.extend(STAGES.map(|stage| u64::from(trace.contains(&stage))));
    values
}

pub fn state(ty: &str, word: u64, first: bool, second: bool) -> Vec<u64> {
    let mut values = vec![word];
    values.extend(arguments(ty, word, first, second));
    values.push(u64::from(oracle(ty, word, first, second).result));
    values
}
