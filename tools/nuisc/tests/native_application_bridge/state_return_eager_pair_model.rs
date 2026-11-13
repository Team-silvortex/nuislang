pub fn execute(input: &[i64], prior: i64, op: &str) -> Result<(i64, Vec<i64>), Vec<i64>> {
    let mut trace = Vec::new();
    let result = if input[0] == 0 {
        trace.push(11);
        false
    } else {
        trace.push(10);
        let left = if input[1] == 0 {
            false
        } else {
            trace.push(20);
            if input[5] == 0 {
                return Err(trace);
            }
            trace.push(21);
            input[3] != 0
        };
        let right = if input[2] != 0 {
            true
        } else {
            trace.push(22);
            if input[6] == 0 {
                return Err(trace);
            }
            trace.push(23);
            input[4] != 0
        };
        let result = match op {
            "==" => left == right,
            "!=" => left != right,
            "^" => left ^ right,
            _ => unreachable!(),
        };
        trace.push(30);
        if input[7] == 0 {
            return Err(trace);
        }
        result
    };
    Ok((prior + if result { 1 } else { 2 }, trace))
}
