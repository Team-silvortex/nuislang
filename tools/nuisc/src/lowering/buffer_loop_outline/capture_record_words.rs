use super::*;
use crate::lowering::scalar_record_shape::{source_value, Shape};

#[path = "capture_record_demand.rs"]
mod demand;

#[cfg(test)]
#[path = "capture_record_words_tests.rs"]
mod tests;

pub(super) struct WordInput {
    param: NirParam,
    shape: Shape,
}

impl WordInput {
    pub(super) fn valid_argument(&self, arg: &NirExpr) -> bool {
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = arg
        else {
            return false;
        };
        let leaves = self.shape.leaves();
        if type_name != &self.param.ty.name || !type_args.is_empty() || fields.len() != leaves.len()
        {
            return false;
        }
        let first = match &fields[0].1 {
            NirExpr::CastBoolToI64(value)
            | NirExpr::CastI32ToI64(value)
            | NirExpr::PackF32Word(value)
            | NirExpr::PackF64Word(value) => value.as_ref(),
            value => value,
        };
        let Some(path) = access(first) else {
            return false;
        };
        // Every word must be the exact codec of a ready leaf of one source.
        // This also proves that removing unused decodes cannot hide a trap.
        fields
            .iter()
            .zip(leaves)
            .enumerate()
            .all(|(index, ((name, value), (leaf, ty)))| {
                name == &format!("carry{index}")
                    && value == &scalar_carries::encode(&ty, source_value(&path[0], &leaf))
            })
    }

    fn reconstruction(&self) -> NirExpr {
        let mut slot = 0;
        self.shape.reconstruct(&mut |ty| {
            let word = source_value(&self.param.name, &[format!("carry{slot}")]);
            slot += 1;
            scalar_carries::decode(ty, word)
        })
    }
}

pub(super) fn normalize(
    function: &mut NirFunction,
    inputs: Option<&scoped_inputs::Inputs>,
    definitions: &BTreeMap<&str, &NirStructDef>,
    layouts: &impl ValueLayouts,
) -> BTreeMap<usize, WordInput> {
    let mut normalized = BTreeMap::new();
    let Some(inputs) = inputs else {
        return normalized;
    };
    if !walk::supported(&function.body) {
        return normalized;
    }
    let mut written = BTreeSet::new();
    branches::collect_bindings(&function.body, &mut written);
    for (index, seed) in &inputs.elidable {
        let Some(param) = function.params.get(*index) else {
            continue;
        };
        if param.ty == *seed.source_type() || written.contains(&param.name) {
            continue;
        }
        let Some(shape) = Shape::from_definitions(seed.source_type(), definitions) else {
            continue;
        };
        let input = WordInput {
            param: param.clone(),
            shape,
        };
        let reconstruction = input.reconstruction();
        let Some((position, name)) =
            function
                .body
                .iter()
                .enumerate()
                .find_map(|(i, stmt)| match stmt {
                    NirStmt::Let {
                        name,
                        ty: Some(ty),
                        value,
                    } if ty == &input.shape.ty && value == &reconstruction => {
                        Some((i, name.clone()))
                    }
                    _ => None,
                })
        else {
            continue;
        };
        let mut tail = function.body[position + 1..].to_vec();
        let mut visible = function
            .params
            .iter()
            .map(|p| p.name.clone())
            .collect::<BTreeSet<_>>();
        // Only preceding bindings in this lexical scope are visible. Child names
        // must neither shadow an outer write nor leak into sibling/parent scopes.
        for stmt in &function.body[..=position] {
            if let NirStmt::Let { name, .. } | NirStmt::Const { name, .. } = stmt {
                visible.insert(name.clone());
            }
        }
        if rewrite_block(
            &mut tail,
            BTreeMap::from([(name.clone(), reconstruction)]),
            visible,
            &BTreeSet::new(),
        )
        .is_none()
        {
            let Some(reconstruction) =
                demand::reconstruction(function, position, &name, &input, definitions, layouts)
            else {
                continue;
            };
            let NirStmt::Let { value, .. } = &mut function.body[position] else {
                unreachable!()
            };
            *value = reconstruction;
        } else {
            function.body.truncate(position);
            function.body.extend(tail);
        }
        normalized.insert(*index, input);
    }
    normalized
}

// Only the proven input reconstruction and ready record aliases may disappear.
// Computed constructors, including their unused checked fields, remain intact.
fn rewrite_block(
    body: &mut Vec<NirStmt>,
    mut origins: BTreeMap<String, NirExpr>,
    mut visible: BTreeSet<String>,
    inherited: &BTreeSet<String>,
) -> Option<()> {
    for mut stmt in std::mem::take(body) {
        match &mut stmt {
            NirStmt::Let { name, ty, value } => {
                visible.insert(name.clone());
                if alias(name, ty.as_ref(), value, &mut origins, inherited)? {
                    continue;
                }
            }
            NirStmt::Const { name, ty, value } => {
                visible.insert(name.clone());
                if alias(name, Some(ty), value, &mut origins, inherited)? {
                    continue;
                }
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                rewrite_value(condition, &origins)?;
                rewrite_block(then_body, origins.clone(), visible.clone(), &visible)?;
                rewrite_block(else_body, origins.clone(), visible.clone(), &visible)?;
            }
            NirStmt::While { condition, body } => {
                // Every removed outer write must preserve the exact immutable
                // origin. Thus zero trips, joins and backedges all see one version.
                rewrite_value(condition, &origins)?;
                rewrite_block(body, origins.clone(), visible.clone(), &visible)?;
            }
            NirStmt::Print(value)
            | NirStmt::Expr(value)
            | NirStmt::Await(value)
            | NirStmt::Return(Some(value)) => rewrite_value(value, &origins)?,
            NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
        }
        body.push(stmt);
    }
    Some(())
}

fn alias(
    name: &str,
    declared: Option<&NirTypeRef>,
    value: &mut NirExpr,
    origins: &mut BTreeMap<String, NirExpr>,
    inherited: &BTreeSet<String>,
) -> Option<bool> {
    let origin =
        access(value).and_then(|path| project(origins.get(&path[0])?, &path[1..]).cloned());
    if let Some(origin @ NirExpr::StructLiteral { .. }) = origin {
        // New child aliases are local. An outer assignment may disappear only
        // when it is already an alias of this identical decoded leaf tree.
        if inherited.contains(name) && origins.get(name) != Some(&origin) {
            return None;
        }
        let NirExpr::StructLiteral { type_name, .. } = &origin else {
            unreachable!()
        };
        if declared.is_none_or(|ty| ty == &scalar_type(type_name)) {
            origins.insert(name.to_owned(), origin);
            return Some(true);
        }
        return None;
    }
    if inherited.contains(name) && origins.contains_key(name) {
        return None;
    }
    rewrite_value(value, origins)?;
    origins.remove(name);
    Some(false)
}

fn rewrite_value(value: &mut NirExpr, origins: &BTreeMap<String, NirExpr>) -> Option<()> {
    let mut valid = true;
    walk::rewrite_expr(value, |value| {
        let Some(path) = access(value) else {
            return;
        };
        let Some(origin) = origins.get(&path[0]) else {
            return;
        };
        match project(origin, &path[1..]) {
            Some(NirExpr::StructLiteral { .. }) | None => valid = false,
            Some(leaf) => *value = leaf.clone(),
        }
    });
    valid.then_some(())
}

pub(super) fn project<'a>(mut value: &'a NirExpr, path: &[String]) -> Option<&'a NirExpr> {
    for field in path {
        let NirExpr::StructLiteral { fields, .. } = value else {
            return None;
        };
        value = &fields.iter().find(|(name, _)| name == field)?.1;
    }
    Some(value)
}
