use yir_core::TensorValue;

pub(crate) fn reshape(
    input: &TensorValue,
    rows: usize,
    cols: usize,
) -> Result<TensorValue, String> {
    if rows == 0 || cols == 0 {
        return Err("kernel.reshape requires non-zero target shape".to_owned());
    }
    if rows * cols != input.elements.len() {
        return Err(format!(
            "kernel.reshape element mismatch: input has {} elements, target shape is {}x{}",
            input.elements.len(),
            rows,
            cols
        ));
    }
    Ok(TensorValue {
        rows,
        cols,
        elements: input.elements.clone(),
    })
}

pub(crate) fn slice(
    input: &TensorValue,
    row_offset: usize,
    col_offset: usize,
    rows: usize,
    cols: usize,
) -> Result<TensorValue, String> {
    if rows == 0 || cols == 0 {
        return Err("kernel.slice requires non-zero slice shape".to_owned());
    }
    if row_offset + rows > input.rows || col_offset + cols > input.cols {
        return Err(format!(
            "kernel.slice out of bounds: {}x{} tensor cannot provide slice ({}, {}) + {}x{}",
            input.rows, input.cols, row_offset, col_offset, rows, cols
        ));
    }

    let mut elements = Vec::with_capacity(rows * cols);
    for row in row_offset..(row_offset + rows) {
        let start = row * input.cols + col_offset;
        let end = start + cols;
        elements.extend_from_slice(&input.elements[start..end]);
    }

    Ok(TensorValue {
        rows,
        cols,
        elements,
    })
}

pub(crate) fn extract_row(input: &TensorValue, row: usize) -> TensorValue {
    let start = row * input.cols;
    let end = start + input.cols;
    TensorValue {
        rows: 1,
        cols: input.cols,
        elements: input.elements[start..end].to_vec(),
    }
}

pub(crate) fn extract_col(input: &TensorValue, col: usize) -> TensorValue {
    TensorValue {
        rows: input.rows,
        cols: 1,
        elements: (0..input.rows)
            .map(|row| input.elements[row * input.cols + col])
            .collect(),
    }
}

pub(crate) fn broadcast(
    input: &TensorValue,
    rows: usize,
    cols: usize,
) -> Result<TensorValue, String> {
    if rows == 0 || cols == 0 {
        return Err("kernel.broadcast requires non-zero target shape".to_owned());
    }
    if input.rows == rows && input.cols == cols {
        return Ok(input.clone());
    }

    let row_compatible = input.rows == 1 || input.rows == rows;
    let col_compatible = input.cols == 1 || input.cols == cols;
    if !row_compatible || !col_compatible {
        return Err(format!(
            "kernel.broadcast shape mismatch: cannot broadcast {}x{} to {}x{}",
            input.rows, input.cols, rows, cols
        ));
    }

    let mut elements = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        let src_row = if input.rows == 1 { 0 } else { row };
        for col in 0..cols {
            let src_col = if input.cols == 1 { 0 } else { col };
            elements.push(input.elements[src_row * input.cols + src_col]);
        }
    }

    Ok(TensorValue {
        rows,
        cols,
        elements,
    })
}

pub(crate) fn reduce_sum_rows(input: &TensorValue) -> TensorValue {
    TensorValue {
        rows: input.rows,
        cols: 1,
        elements: (0..input.rows)
            .map(|row| {
                let start = row * input.cols;
                let end = start + input.cols;
                input.elements[start..end].iter().copied().sum()
            })
            .collect(),
    }
}

pub(crate) fn reduce_max_rows(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.rows)
        .map(|row| {
            let start = row * input.cols;
            let end = start + input.cols;
            input.elements[start..end]
                .iter()
                .copied()
                .max()
                .ok_or_else(|| "kernel.reduce_max_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: input.rows,
        cols: 1,
        elements,
    })
}

pub(crate) fn reduce_min_rows(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.rows)
        .map(|row| {
            let start = row * input.cols;
            let end = start + input.cols;
            input.elements[start..end]
                .iter()
                .copied()
                .min()
                .ok_or_else(|| "kernel.reduce_min_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: input.rows,
        cols: 1,
        elements,
    })
}

pub(crate) fn reduce_sum_cols(input: &TensorValue) -> TensorValue {
    TensorValue {
        rows: 1,
        cols: input.cols,
        elements: (0..input.cols)
            .map(|col| {
                (0..input.rows)
                    .map(|row| input.elements[row * input.cols + col])
                    .sum()
            })
            .collect(),
    }
}

pub(crate) fn reduce_max_cols(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.cols)
        .map(|col| {
            (0..input.rows)
                .map(|row| input.elements[row * input.cols + col])
                .max()
                .ok_or_else(|| "kernel.reduce_max_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: 1,
        cols: input.cols,
        elements,
    })
}

pub(crate) fn reduce_min_cols(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.cols)
        .map(|col| {
            (0..input.rows)
                .map(|row| input.elements[row * input.cols + col])
                .min()
                .ok_or_else(|| "kernel.reduce_min_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: 1,
        cols: input.cols,
        elements,
    })
}

pub(crate) fn reduce_mean_rows(input: &TensorValue) -> TensorValue {
    TensorValue {
        rows: input.rows,
        cols: 1,
        elements: (0..input.rows)
            .map(|row| {
                let start = row * input.cols;
                let end = start + input.cols;
                let sum: i64 = input.elements[start..end].iter().copied().sum();
                sum / input.cols as i64
            })
            .collect(),
    }
}

pub(crate) fn reduce_mean_cols(input: &TensorValue) -> TensorValue {
    TensorValue {
        rows: 1,
        cols: input.cols,
        elements: (0..input.cols)
            .map(|col| {
                let sum: i64 = (0..input.rows)
                    .map(|row| input.elements[row * input.cols + col])
                    .sum();
                sum / input.rows as i64
            })
            .collect(),
    }
}

pub(crate) fn argmax_rows(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.rows)
        .map(|row| {
            let start = row * input.cols;
            let end = start + input.cols;
            input.elements[start..end]
                .iter()
                .copied()
                .enumerate()
                .max_by_key(|(_, value)| *value)
                .map(|(index, _)| index as i64)
                .ok_or_else(|| "kernel.argmax_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: input.rows,
        cols: 1,
        elements,
    })
}

pub(crate) fn argmax_cols(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.cols)
        .map(|col| {
            (0..input.rows)
                .map(|row| (row, input.elements[row * input.cols + col]))
                .max_by_key(|(_, value)| *value)
                .map(|(row, _)| row as i64)
                .ok_or_else(|| "kernel.argmax_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: 1,
        cols: input.cols,
        elements,
    })
}

pub(crate) fn argmin_rows(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.rows)
        .map(|row| {
            let start = row * input.cols;
            let end = start + input.cols;
            input.elements[start..end]
                .iter()
                .copied()
                .enumerate()
                .min_by_key(|(_, value)| *value)
                .map(|(index, _)| index as i64)
                .ok_or_else(|| "kernel.argmin_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: input.rows,
        cols: 1,
        elements,
    })
}

pub(crate) fn argmin_cols(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.cols)
        .map(|col| {
            (0..input.rows)
                .map(|row| (row, input.elements[row * input.cols + col]))
                .min_by_key(|(_, value)| *value)
                .map(|(row, _)| row as i64)
                .ok_or_else(|| "kernel.argmin_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: 1,
        cols: input.cols,
        elements,
    })
}

pub(crate) fn sort_tensor_flat(input: &TensorValue) -> TensorValue {
    let mut elements = input.elements.clone();
    elements.sort_unstable();
    TensorValue {
        rows: 1,
        cols: elements.len(),
        elements,
    }
}

pub(crate) fn topk_tensor_flat(input: &TensorValue, k: usize) -> Result<TensorValue, String> {
    if k == 0 {
        return Err("kernel.topk requires k > 0".to_owned());
    }
    if k > input.elements.len() {
        return Err(format!(
            "kernel.topk requested {} values from tensor with only {} elements",
            k,
            input.elements.len()
        ));
    }
    let mut elements = input.elements.clone();
    elements.sort_unstable_by(|lhs, rhs| rhs.cmp(lhs));
    elements.truncate(k);
    Ok(TensorValue {
        rows: 1,
        cols: k,
        elements,
    })
}

pub(crate) fn topk_rows(input: &TensorValue, k: usize) -> Result<TensorValue, String> {
    if k == 0 {
        return Err("kernel.topk_axis requires k > 0".to_owned());
    }
    if k > input.cols {
        return Err(format!(
            "kernel.topk_axis requested top-{} across rows of width {}",
            k, input.cols
        ));
    }
    let mut elements = Vec::with_capacity(input.rows * k);
    for row in 0..input.rows {
        let start = row * input.cols;
        let end = start + input.cols;
        let mut row_values = input.elements[start..end].to_vec();
        row_values.sort_unstable_by(|lhs, rhs| rhs.cmp(lhs));
        row_values.truncate(k);
        elements.extend(row_values);
    }
    Ok(TensorValue {
        rows: input.rows,
        cols: k,
        elements,
    })
}

pub(crate) fn topk_cols(input: &TensorValue, k: usize) -> Result<TensorValue, String> {
    if k == 0 {
        return Err("kernel.topk_axis requires k > 0".to_owned());
    }
    if k > input.rows {
        return Err(format!(
            "kernel.topk_axis requested top-{} across cols of height {}",
            k, input.rows
        ));
    }
    let mut columns = Vec::with_capacity(input.cols * k);
    for col in 0..input.cols {
        let mut col_values = (0..input.rows)
            .map(|row| input.elements[row * input.cols + col])
            .collect::<Vec<_>>();
        col_values.sort_unstable_by(|lhs, rhs| rhs.cmp(lhs));
        col_values.truncate(k);
        columns.push(col_values);
    }

    let mut elements = Vec::with_capacity(k * input.cols);
    for rank in 0..k {
        for column in &columns {
            elements.push(column[rank]);
        }
    }
    Ok(TensorValue {
        rows: k,
        cols: input.cols,
        elements,
    })
}

pub(crate) fn sort_rows(input: &TensorValue) -> TensorValue {
    let mut elements = Vec::with_capacity(input.elements.len());
    for row in 0..input.rows {
        let start = row * input.cols;
        let end = start + input.cols;
        let mut row_values = input.elements[start..end].to_vec();
        row_values.sort_unstable();
        elements.extend(row_values);
    }
    TensorValue {
        rows: input.rows,
        cols: input.cols,
        elements,
    }
}

pub(crate) fn sort_cols(input: &TensorValue) -> TensorValue {
    let mut columns = Vec::with_capacity(input.cols);
    for col in 0..input.cols {
        let mut col_values = (0..input.rows)
            .map(|row| input.elements[row * input.cols + col])
            .collect::<Vec<_>>();
        col_values.sort_unstable();
        columns.push(col_values);
    }

    let mut elements = Vec::with_capacity(input.elements.len());
    for row in 0..input.rows {
        for column in &columns {
            elements.push(column[row]);
        }
    }
    TensorValue {
        rows: input.rows,
        cols: input.cols,
        elements,
    }
}

pub(crate) fn matmul(lhs: &TensorValue, rhs: &TensorValue) -> Result<TensorValue, String> {
    if lhs.cols != rhs.rows {
        return Err(format!(
            "kernel.matmul shape mismatch: lhs is {}x{}, rhs is {}x{}",
            lhs.rows, lhs.cols, rhs.rows, rhs.cols
        ));
    }

    let mut elements = vec![0i64; lhs.rows * rhs.cols];
    for row in 0..lhs.rows {
        for col in 0..rhs.cols {
            let mut acc = 0i64;
            for k in 0..lhs.cols {
                acc += lhs.elements[row * lhs.cols + k] * rhs.elements[k * rhs.cols + col];
            }
            elements[row * rhs.cols + col] = acc;
        }
    }

    Ok(TensorValue {
        rows: lhs.rows,
        cols: rhs.cols,
        elements,
    })
}

pub(crate) fn add_bias(input: &TensorValue, bias: &TensorValue) -> Result<TensorValue, String> {
    let broadcasted = broadcast(bias, input.rows, input.cols).map_err(|_| {
        format!(
            "kernel.add_bias shape mismatch: input is {}x{}, bias is {}x{}",
            input.rows, input.cols, bias.rows, bias.cols
        )
    })?;
    Ok(TensorValue {
        rows: input.rows,
        cols: input.cols,
        elements: input
            .elements
            .iter()
            .copied()
            .zip(broadcasted.elements)
            .map(|(lhs, rhs)| lhs + rhs)
            .collect(),
    })
}

pub(crate) fn transpose(input: &TensorValue) -> TensorValue {
    let mut elements = vec![0; input.rows * input.cols];
    for row in 0..input.rows {
        for col in 0..input.cols {
            elements[col * input.rows + row] = input.elements[row * input.cols + col];
        }
    }

    TensorValue {
        rows: input.cols,
        cols: input.rows,
        elements,
    }
}
