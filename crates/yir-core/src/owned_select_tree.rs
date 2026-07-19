use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnedSelectScalarCast {
    I64ToI32,
    I32ToI64,
    I64ToBool,
    BoolToI64,
    I64ToF32,
    F32ToI64,
    I64ToF64,
    F64ToI64,
}

impl OwnedSelectScalarCast {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "i64_to_i32" => Self::I64ToI32,
            "i32_to_i64" => Self::I32ToI64,
            "i64_to_bool" => Self::I64ToBool,
            "bool_to_i64" => Self::BoolToI64,
            "i64_to_f32" => Self::I64ToF32,
            "f32_to_i64" => Self::F32ToI64,
            "i64_to_f64" => Self::I64ToF64,
            "f64_to_i64" => Self::F64ToI64,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::I64ToI32 => "i64_to_i32",
            Self::I32ToI64 => "i32_to_i64",
            Self::I64ToBool => "i64_to_bool",
            Self::BoolToI64 => "bool_to_i64",
            Self::I64ToF32 => "i64_to_f32",
            Self::F32ToI64 => "f32_to_i64",
            Self::I64ToF64 => "i64_to_f64",
            Self::F64ToI64 => "f64_to_i64",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedSelectScalarArg<'a> {
    Value(&'a str),
    VariantField {
        base: &'a str,
        variant: &'a str,
        field: &'a str,
    },
    StructField {
        field: &'a str,
        base: Box<OwnedSelectScalarArg<'a>>,
    },
    Cast {
        kind: OwnedSelectScalarCast,
        value: Box<OwnedSelectScalarArg<'a>>,
    },
    NonNull {
        value: Box<OwnedSelectScalarArg<'a>>,
    },
    TraversalBorrow {
        value: Box<OwnedSelectScalarArg<'a>>,
    },
    OwnedTransfer {
        value: &'a str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedSelectTree<'a> {
    Owner(usize),
    Call {
        callee: &'a str,
        owner: usize,
        scalar_args: Vec<OwnedSelectScalarArg<'a>>,
    },
    If {
        condition: &'a str,
        then_tree: Box<OwnedSelectTree<'a>>,
        else_tree: Box<OwnedSelectTree<'a>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedSelectTreeArgs<'a> {
    pub owners: &'a [String],
    pub tree: OwnedSelectTree<'a>,
}

pub fn parse_owned_select_tree_args(args: &[String]) -> Option<OwnedSelectTreeArgs<'_>> {
    let owner_count = args.first()?.parse::<usize>().ok()?;
    let owner_end = 1usize.checked_add(owner_count)?;
    let owners = args.get(1..owner_end)?;
    if owners.is_empty() {
        return None;
    }
    let mut cursor = owner_end;
    let tree = parse_tree(args, &mut cursor, owners.len())?;
    if cursor != args.len() || owned_transfer_set(&tree).is_none() {
        return None;
    }
    Some(OwnedSelectTreeArgs { owners, tree })
}

fn parse_tree<'a>(
    args: &'a [String],
    cursor: &mut usize,
    owner_count: usize,
) -> Option<OwnedSelectTree<'a>> {
    let tag = args.get(*cursor)?;
    *cursor += 1;
    match tag.as_str() {
        "owner" => {
            let index = args.get(*cursor)?.parse::<usize>().ok()?;
            *cursor += 1;
            (index < owner_count).then_some(OwnedSelectTree::Owner(index))
        }
        "call" => {
            let callee = args.get(*cursor)?;
            let owner = args.get(cursor.checked_add(1)?)?.parse::<usize>().ok()?;
            let scalar_count = args.get(cursor.checked_add(2)?)?.parse::<usize>().ok()?;
            *cursor = cursor.checked_add(3)?;
            let mut scalar_args = Vec::with_capacity(scalar_count);
            for _ in 0..scalar_count {
                scalar_args.push(parse_scalar_arg(args, cursor)?);
            }
            (owner < owner_count).then_some(OwnedSelectTree::Call {
                callee,
                owner,
                scalar_args,
            })
        }
        "if" => {
            let condition = args.get(*cursor)?;
            *cursor += 1;
            let then_tree = parse_tree(args, cursor, owner_count)?;
            let else_tree = parse_tree(args, cursor, owner_count)?;
            Some(OwnedSelectTree::If {
                condition,
                then_tree: Box::new(then_tree),
                else_tree: Box::new(else_tree),
            })
        }
        _ => None,
    }
}

fn parse_scalar_arg<'a>(
    args: &'a [String],
    cursor: &mut usize,
) -> Option<OwnedSelectScalarArg<'a>> {
    let tag = args.get(*cursor)?;
    *cursor += 1;
    match tag.as_str() {
        "value" => {
            let value = args.get(*cursor)?;
            *cursor += 1;
            Some(OwnedSelectScalarArg::Value(value))
        }
        "variant_field" => {
            let base = args.get(*cursor)?;
            let variant = args.get(cursor.checked_add(1)?)?;
            let field = args.get(cursor.checked_add(2)?)?;
            *cursor = cursor.checked_add(3)?;
            Some(OwnedSelectScalarArg::VariantField {
                base,
                variant,
                field,
            })
        }
        "struct_field" => {
            let field = args.get(*cursor)?;
            *cursor += 1;
            let base = parse_scalar_arg(args, cursor)?;
            Some(OwnedSelectScalarArg::StructField {
                field,
                base: Box::new(base),
            })
        }
        "cast" => {
            let kind = OwnedSelectScalarCast::parse(args.get(*cursor)?)?;
            *cursor += 1;
            let value = parse_scalar_arg(args, cursor)?;
            Some(OwnedSelectScalarArg::Cast {
                kind,
                value: Box::new(value),
            })
        }
        "non_null" => {
            let value = parse_scalar_arg(args, cursor)?;
            Some(OwnedSelectScalarArg::NonNull {
                value: Box::new(value),
            })
        }
        "traversal_borrow" => {
            let value = parse_scalar_arg(args, cursor)?;
            Some(OwnedSelectScalarArg::TraversalBorrow {
                value: Box::new(value),
            })
        }
        "owned_transfer" => {
            let value = args.get(*cursor)?;
            *cursor += 1;
            Some(OwnedSelectScalarArg::OwnedTransfer { value })
        }
        _ => None,
    }
}

pub fn owned_select_tree_scalar_args<'a>(tree: &'a OwnedSelectTree<'a>, out: &mut Vec<&'a str>) {
    match tree {
        OwnedSelectTree::Owner(_) => {}
        OwnedSelectTree::Call { scalar_args, .. } => {
            for arg in scalar_args {
                owned_select_scalar_arg_inputs(arg, out);
            }
        }
        OwnedSelectTree::If {
            then_tree,
            else_tree,
            ..
        } => {
            owned_select_tree_scalar_args(then_tree, out);
            owned_select_tree_scalar_args(else_tree, out);
        }
    }
}

fn owned_select_scalar_arg_inputs<'a>(arg: &'a OwnedSelectScalarArg<'a>, out: &mut Vec<&'a str>) {
    match arg {
        OwnedSelectScalarArg::Value(value) => out.push(value),
        OwnedSelectScalarArg::VariantField { base, .. } => out.push(base),
        OwnedSelectScalarArg::StructField { base, .. } => {
            owned_select_scalar_arg_inputs(base, out);
        }
        OwnedSelectScalarArg::Cast { value, .. } => owned_select_scalar_arg_inputs(value, out),
        OwnedSelectScalarArg::NonNull { value } => owned_select_scalar_arg_inputs(value, out),
        OwnedSelectScalarArg::TraversalBorrow { value } => {
            owned_select_scalar_arg_inputs(value, out)
        }
        OwnedSelectScalarArg::OwnedTransfer { value } => out.push(value),
    }
}

pub fn owned_select_tree_transfers<'a>(tree: &'a OwnedSelectTree<'a>, out: &mut Vec<&'a str>) {
    if let Some(transfers) = owned_transfer_set(tree) {
        out.extend(transfers);
    }
}

fn owned_transfer_set<'a>(tree: &'a OwnedSelectTree<'a>) -> Option<BTreeSet<&'a str>> {
    match tree {
        OwnedSelectTree::Owner(_) => Some(BTreeSet::new()),
        OwnedSelectTree::Call { scalar_args, .. } => {
            let mut transfers = BTreeSet::new();
            for arg in scalar_args {
                if let OwnedSelectScalarArg::OwnedTransfer { value } = arg {
                    if !transfers.insert(*value) {
                        return None;
                    }
                }
            }
            Some(transfers)
        }
        OwnedSelectTree::If {
            then_tree,
            else_tree,
            ..
        } => {
            let then_transfers = owned_transfer_set(then_tree)?;
            let else_transfers = owned_transfer_set(else_tree)?;
            (then_transfers == else_transfers).then_some(then_transfers)
        }
    }
}

pub fn owned_select_tree_conditions<'a>(tree: &'a OwnedSelectTree<'a>, out: &mut Vec<&'a str>) {
    if let OwnedSelectTree::If {
        condition,
        then_tree,
        else_tree,
    } = tree
    {
        out.push(condition);
        owned_select_tree_conditions(then_tree, out);
        owned_select_tree_conditions(else_tree, out);
    }
}
