// Independent imperative replay uses signed i32 values, never the compiler codec.
pub fn oracle(seed: i64, first: bool, second: bool) -> (bool, Vec<i32>) {
    let mut trace = vec![99, 90, 87];
    let mut local = seed < 0;
    let result;
    if first {
        if second {
            trace.extend([91, 92]);
            let mut child = seed as i32;
            if local {
                child = !child;
                trace.push(93);
                local = child == seed as i32;
                trace.push(95);
                let _ = local;
                if child < 0 {
                    local = true;
                    trace.push(93);
                } else {
                    local = false;
                }
                local = local || false;
            } else {
                local = child == seed as i32;
                trace.push(95);
                trace.push(93);
                if local {
                    child = !child;
                    trace.push(93);
                    local = child != seed as i32;
                }
                local = local && first;
            }
            if local {
                child = !child;
                trace.push(93);
                local = child == seed as i32;
            } else {
                local = child != seed as i32;
            }
            // Read the final carried child after every join, not the pre-join seed.
            let _ = local;
            trace.push(96);
            local = child == seed as i32;
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
    (result, trace)
}
