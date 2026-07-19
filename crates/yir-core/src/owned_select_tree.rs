#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedSelectTree<'a> {
    Owner(usize),
    Call {
        callee: &'a str,
        owner: usize,
        scalar_args: &'a [String],
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
    (cursor == args.len()).then_some(OwnedSelectTreeArgs { owners, tree })
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
            let scalar_end = cursor.checked_add(scalar_count)?;
            let scalar_args = args.get(*cursor..scalar_end)?;
            *cursor = scalar_end;
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

pub fn owned_select_tree_scalar_args<'a>(tree: &'a OwnedSelectTree<'a>, out: &mut Vec<&'a str>) {
    match tree {
        OwnedSelectTree::Owner(_) => {}
        OwnedSelectTree::Call { scalar_args, .. } => {
            out.extend(scalar_args.iter().map(String::as_str));
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
