// Independent imperative replay: nested branch-local sequences observe
// their own preceding writes and only the two current versions cross a join.
pub fn oracle(seed: bool, first: bool, second: bool, reply: bool) -> (bool, Vec<i32>) {
    let mut local = seed;
    let mut trace = vec![99, 90, 87];
    let result;
    if first {
        if second {
            trace.extend([91, 92]);
            let mut child = local;
            if reply {
                child = if child {
                    true
                } else {
                    trace.push(93);
                    reply
                };
                local = if child {
                    trace.push(95);
                    seed
                } else {
                    false
                };
                let current = child;
                trace.push(88);
                if current {
                    child = if child {
                        trace.push(94);
                        local
                    } else {
                        false
                    };
                    local = if child {
                        true
                    } else {
                        trace.push(86);
                        seed
                    };
                }
                if local {
                    child = child || reply;
                } else {
                    local = child && second;
                }
                local = local || child;
            } else {
                local = if child {
                    true
                } else {
                    trace.push(95);
                    second
                };
                child = if local {
                    trace.push(93);
                    reply
                } else {
                    false
                };
                let local_seal = local || false;
                trace.push(88);
                if local_seal {
                    local = if local {
                        trace.push(86);
                        child
                    } else {
                        false
                    };
                    child = if local {
                        true
                    } else {
                        trace.push(94);
                        second
                    };
                }
                if child {
                    local = child && seed;
                } else {
                    child = local || second;
                }
                local = local || child;
            }
            if child {
                local = if local {
                    trace.push(96);
                    child
                } else {
                    false
                };
                child = local && seed;
            } else {
                child = if child {
                    true
                } else {
                    trace.push(96);
                    local
                };
                local = child || seed;
            }
            if local {
                child = child || second;
            } else {
                local = child && seed;
            }
            let seal = child || false;
            local = local && seal;
            trace.push(83);
        }
        result = if local {
            trace.push(97);
            reply
        } else {
            false
        };
    } else {
        result = if local {
            true
        } else {
            trace.push(97);
            seed
        };
    }
    trace.push(79);
    (result, trace)
}

pub const STAGES: [i32; 12] = [90, 87, 91, 92, 93, 95, 96, 83, 97, 88, 94, 86];
