use super::*;
use crate::lowering::scalar_record_shape::{source_value, Shape};
use nuis_semantics::model::NirStructField;

#[path = "return_invariant_facts.rs"]
mod facts;

#[cfg(test)]
#[path = "return_invariants_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "return_invariant_nested_tests.rs"]
mod nested_tests;

#[cfg(test)]
#[path = "return_invariant_admission_tests.rs"]
mod admission_tests;

#[cfg(test)]
#[path = "return_invariant_child_tests.rs"]
mod child_tests;

#[cfg(test)]
#[path = "return_invariant_post_loop_tests.rs"]
mod post_loop_tests;

#[cfg(test)]
#[path = "return_invariant_literal_execution_tests.rs"]
mod literal_execution_tests;

#[cfg(test)]
#[path = "return_invariant_joins_tests.rs"]
mod joins_tests;

const MAX_WORK: usize = 65_536;
const MAX_DEPTH: usize = 64;

struct Budget(usize);

impl Budget {
    fn charge(&mut self, work: usize) -> Option<()> {
        self.0 = self.0.checked_sub(work)?;
        Some(())
    }

    fn tick(&mut self, depth: usize) -> Option<()> {
        if depth >= MAX_DEPTH {
            return None;
        }
        self.charge(1)
    }
}

pub(super) fn rewrite(
    function: &NirFunction,
    body: &[NirStmt],
    signal: &Signal,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
    reserved: &BTreeSet<String>,
    nested: bool,
) -> Option<(Vec<NirStmt>, Vec<NirStructDef>)> {
    let scope: Scope = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let mut names = reserved.clone();
    names.extend(scope.keys().cloned());
    names.extend(layouts.keys().cloned());
    branches::collect_bindings(body, &mut names);
    reserve_references(body, &mut names)?;
    Pass {
        signal,
        layouts: layouts.clone(),
        catalog,
        names,
        budget: Budget(MAX_WORK),
        changed: false,
        structs: Vec::new(),
        nested,
        snapshot_clock: 0,
    }
    .run(body, scope)
}

struct Record {
    name: String,
    shape: Shape,
    slots: Vec<Option<String>>,
    carry: Option<(String, NirTypeRef)>,
}

struct Pass<'a> {
    signal: &'a Signal,
    layouts: control_values::CarryLayouts,
    catalog: &'a ScalarHelpers,
    names: BTreeSet<String>,
    budget: Budget,
    changed: bool,
    structs: Vec<NirStructDef>,
    nested: bool,
    snapshot_clock: usize,
}

impl Pass<'_> {
    fn run(&mut self, body: &[NirStmt], scope: Scope) -> Option<(Vec<NirStmt>, Vec<NirStructDef>)> {
        let snapshots =
            facts::Snapshots::new(&scope, &self.layouts, self.catalog, &mut self.budget)?;
        let (output, _) = self.block(body, scope, snapshots, 0)?;
        self.changed
            .then(|| (output, std::mem::take(&mut self.structs)))
    }

    fn record(&mut self, name: String, shape: Shape, stable: Vec<bool>) -> Record {
        let mut fields = Vec::new();
        let slots = shape
            .leaves()
            .into_iter()
            .zip(stable)
            .map(|((_, ty), stable)| {
                if stable {
                    return None;
                }
                let name = format!("field{}", fields.len());
                fields.push(NirStructField {
                    visibility: NirVisibility::Private,
                    annotations: vec![],
                    name: name.clone(),
                    ty,
                });
                Some(name)
            })
            .collect();
        let carry = if fields.is_empty() {
            None
        } else {
            // Keep mutable leaves in a typed record so wide private inputs can
            // reuse the existing complete seed-map transport, not many scalars.
            let ty = branches::fresh_name("__nuis_loop_fields", &mut self.names);
            self.layouts.insert(
                ty.clone(),
                fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.clone()))
                    .collect(),
            );
            self.structs.push(NirStructDef {
                visibility: NirVisibility::Private,
                annotations: vec![],
                name: ty.clone(),
                generic_params: vec![],
                where_bounds: vec![],
                fields,
            });
            Some((
                branches::fresh_name("__nuis_loop_mutable", &mut self.names),
                scalar_type(&ty),
            ))
        };
        Record {
            name,
            shape,
            slots,
            carry,
        }
    }

    fn mutable_value(&self, record: &Record, source: &str) -> Option<NirExpr> {
        let (_, ty) = record.carry.as_ref()?;
        Some(NirExpr::StructLiteral {
            type_name: ty.name.clone(),
            type_args: vec![],
            fields: record
                .shape
                .leaves()
                .iter()
                .zip(&record.slots)
                .filter_map(|((path, _), slot)| Some((slot.clone()?, source_value(source, path))))
                .collect(),
        })
    }

    fn block(
        &mut self,
        body: &[NirStmt],
        mut scope: Scope,
        mut snapshots: facts::Snapshots,
        depth: usize,
    ) -> Option<(Vec<NirStmt>, facts::Snapshots)> {
        let mut output = Vec::new();
        for stmt in body {
            self.budget.tick(depth)?;
            match stmt {
                NirStmt::While { condition, body } => {
                    self.budget.charge(scope.len())?;
                    let loop_facts = if self.nested {
                        Some(snapshots.loop_facts(
                            body,
                            &self.layouts,
                            self.catalog,
                            &mut self.budget,
                            depth + 1,
                        )?)
                    } else {
                        None
                    };
                    let child_snapshots = if self.nested && contains_loop(body) {
                        loop_facts
                            .as_ref()?
                            .at_boundary(&mut self.budget, &mut self.snapshot_clock)?
                    } else {
                        let mut entries = snapshots.clone();
                        entries.forget_writes(
                            body,
                            &mut self.budget,
                            &mut self.snapshot_clock,
                            depth + 1,
                        )?;
                        entries
                    };
                    let (rewritten, _) =
                        self.block(body, scope.clone(), child_snapshots, depth + 1)?;
                    let iteration = induction::parse(condition, &rewritten)?;
                    // Per-write invariance does not depend on exit ownership.
                    // Ordinary exits keep their own control/index state; only
                    // the legacy fallback remains innermost-return-only.
                    let records = if self.nested
                        || (!contains_loop(&rewritten)
                            && exits::return_owned(iteration.effects, self.signal))
                    {
                        let stable = facts::analyze_at(
                            iteration.effects,
                            &scope,
                            &self.layouts,
                            self.catalog,
                            &mut self.budget,
                            Some(&snapshots),
                        )?;
                        stable
                            .into_iter()
                            .map(|(name, (shape, stable))| self.record(name, shape, stable))
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    };
                    if let Some(facts) = &loop_facts {
                        snapshots =
                            facts.at_boundary(&mut self.budget, &mut self.snapshot_clock)?;
                    } else {
                        snapshots.forget_writes(
                            body,
                            &mut self.budget,
                            &mut self.snapshot_clock,
                            depth + 1,
                        )?;
                    }
                    if records.is_empty() {
                        output.push(NirStmt::While {
                            condition: condition.clone(),
                            body: rewritten,
                        });
                        continue;
                    }
                    let mut prefix = Vec::new();
                    for record in &records {
                        self.budget
                            .charge(record.shape.leaves().iter().map(|(p, _)| p.len() + 3).sum())?;
                        if let Some((name, ty)) = &record.carry {
                            output.push(binding(
                                name,
                                ty,
                                self.mutable_value(record, &record.name)?,
                            ));
                            scope.insert(name.clone(), ty.clone());
                            // Self-copies retain the existing ordered-read admission.
                            prefix.push(binding(name, ty, NirExpr::Var(name.clone())));
                        }
                    }
                    let mut effects = self.effects(iteration.effects, &records, depth + 1)?;
                    prefix.append(&mut effects);
                    if iteration.leading {
                        prefix.insert(0, iteration.step.clone());
                    } else {
                        prefix.push(iteration.step.clone());
                    }
                    output.push(NirStmt::While {
                        condition: condition.clone(),
                        body: prefix,
                    });
                    for record in &records {
                        output.push(binding(
                            &record.name,
                            &record.shape.ty,
                            self.read(record, &[], 0)?,
                        ));
                    }
                    self.changed = true;
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    self.budget.charge(scope.len().checked_mul(2)?)?;
                    let condition = self.expression(condition, &[], depth + 1)?;
                    let (then_body, left) =
                        self.block(then_body, scope.clone(), snapshots.clone(), depth + 1)?;
                    let (else_body, right) =
                        self.block(else_body, scope.clone(), snapshots.clone(), depth + 1)?;
                    output.push(NirStmt::If {
                        condition,
                        then_body,
                        else_body,
                    });
                    snapshots.join(&left, &right, &mut self.budget, &mut self.snapshot_clock)?;
                }
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    snapshots.bind(
                        stmt,
                        &self.layouts,
                        self.catalog,
                        &mut self.budget,
                        &mut self.snapshot_clock,
                        depth + 1,
                    )?;
                    scope.insert(
                        name.clone(),
                        control_values::value_type(value, &scope, self.catalog, &self.layouts)?,
                    );
                    output.extend(self.effects(std::slice::from_ref(stmt), &[], depth + 1)?);
                }
                NirStmt::Return(value) => output.push(NirStmt::Return(match value {
                    Some(value) => Some(self.expression(value, &[], depth + 1)?),
                    None => None,
                })),
                _ => output.extend(self.effects(std::slice::from_ref(stmt), &[], depth + 1)?),
            }
        }
        Some((output, snapshots))
    }

    fn effects(
        &mut self,
        body: &[NirStmt],
        records: &[Record],
        depth: usize,
    ) -> Option<Vec<NirStmt>> {
        let mut output = Vec::new();
        for stmt in body {
            self.budget.tick(depth)?;
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    let value = self.expression(value, records, depth + 1)?;
                    if let Some(record) = records.iter().find(|r| &r.name == name) {
                        if let Some((name, ty)) = &record.carry {
                            // A child backedge resets ordered-read availability. Keep
                            // this assignment's own-read authority at its original site,
                            // before splitting the RHS snapshot from field publication.
                            self.budget.charge(2)?;
                            output.push(binding(name, ty, NirExpr::Var(name.clone())));
                        }
                        // Preserve the complete original RHS once, before any leaf publication.
                        let temporary =
                            branches::fresh_name("__nuis_loop_snapshot", &mut self.names);
                        output.push(binding(&temporary, &record.shape.ty, value));
                        self.budget
                            .charge(record.shape.leaves().iter().map(|(p, _)| p.len() + 1).sum())?;
                        if let Some((name, ty)) = &record.carry {
                            output.push(binding(name, ty, self.mutable_value(record, &temporary)?));
                        }
                    } else {
                        output.push(NirStmt::Let {
                            name: name.clone(),
                            ty: ty.clone(),
                            value,
                        });
                    }
                }
                NirStmt::Const { name, ty, value } => output.push(NirStmt::Const {
                    name: name.clone(),
                    ty: ty.clone(),
                    value: self.expression(value, records, depth + 1)?,
                }),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => output.push(NirStmt::If {
                    condition: self.expression(condition, records, depth + 1)?,
                    then_body: self.effects(then_body, records, depth + 1)?,
                    else_body: self.effects(else_body, records, depth + 1)?,
                }),
                NirStmt::While { condition, body } => output.push(NirStmt::While {
                    condition: self.expression(condition, records, depth + 1)?,
                    body: self.effects(body, records, depth + 1)?,
                }),
                NirStmt::Expr(value) => {
                    output.push(NirStmt::Expr(self.expression(value, records, depth + 1)?))
                }
                NirStmt::Break | NirStmt::Continue => output.push(stmt.clone()),
                _ => return None,
            }
        }
        Some(output)
    }

    fn read(&mut self, record: &Record, path: &[String], depth: usize) -> Option<NirExpr> {
        self.budget.tick(depth)?;
        let mut shape = &record.shape;
        for field in path {
            shape = &shape.fields.iter().find(|(name, _)| name == field)?.1;
        }
        let leaves = record.shape.leaves();
        self.budget.charge(leaves.len())?;
        if leaves
            .iter()
            .zip(&record.slots)
            .filter(|((leaf, _), _)| leaf.starts_with(path))
            .all(|(_, slot)| slot.is_none())
        {
            return Some(source_value(&record.name, path));
        }
        let mut values = Vec::new();
        for (leaf, _) in shape.leaves() {
            let mut full = path.to_vec();
            full.extend(leaf);
            self.budget.charge(full.len() + 2)?;
            let index = leaves.iter().position(|(path, _)| path == &full)?;
            values.push(record.slots[index].as_ref().map_or_else(
                || source_value(&record.name, &full),
                |name| {
                    source_value(
                        &record.carry.as_ref().expect("mutable leaf has a record").0,
                        std::slice::from_ref(name),
                    )
                },
            ));
        }
        let mut values = values.into_iter();
        Some(shape.reconstruct(&mut |_| values.next().expect("complete record leaf map")))
    }

    fn expression(&mut self, expr: &NirExpr, records: &[Record], depth: usize) -> Option<NirExpr> {
        self.budget.tick(depth)?;
        let mut base = expr;
        let mut path = Vec::new();
        while let NirExpr::FieldAccess {
            base: parent,
            field,
        } = base
        {
            self.budget.tick(depth + path.len())?;
            path.push(field.clone());
            base = parent;
        }
        if let NirExpr::Var(name) = base {
            if let Some(record) = records.iter().find(|record| &record.name == name) {
                path.reverse();
                return self.read(record, &path, depth + 1);
            }
        }
        Some(match expr {
            NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
                op: *op,
                lhs: Box::new(self.expression(lhs, records, depth + 1)?),
                rhs: Box::new(self.expression(rhs, records, depth + 1)?),
            },
            NirExpr::Call { callee, args } => NirExpr::Call {
                callee: callee.clone(),
                args: args
                    .iter()
                    .map(|arg| self.expression(arg, records, depth + 1))
                    .collect::<Option<_>>()?,
            },
            NirExpr::StructLiteral {
                type_name,
                type_args,
                fields,
            } => NirExpr::StructLiteral {
                type_name: type_name.clone(),
                type_args: type_args.clone(),
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        Some((name.clone(), self.expression(value, records, depth + 1)?))
                    })
                    .collect::<Option<_>>()?,
            },
            NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
                base: Box::new(self.expression(base, records, depth + 1)?),
                field: field.clone(),
            },
            NirExpr::CastI64ToI32(v) => {
                NirExpr::CastI64ToI32(Box::new(self.expression(v, records, depth + 1)?))
            }
            NirExpr::CastI32ToI64(v) => {
                NirExpr::CastI32ToI64(Box::new(self.expression(v, records, depth + 1)?))
            }
            NirExpr::PackF32Word(v) => {
                NirExpr::PackF32Word(Box::new(self.expression(v, records, depth + 1)?))
            }
            NirExpr::UnpackF32Word(v) => {
                NirExpr::UnpackF32Word(Box::new(self.expression(v, records, depth + 1)?))
            }
            NirExpr::PackF64Word(v) => {
                NirExpr::PackF64Word(Box::new(self.expression(v, records, depth + 1)?))
            }
            NirExpr::UnpackF64Word(v) => {
                NirExpr::UnpackF64Word(Box::new(self.expression(v, records, depth + 1)?))
            }
            NirExpr::Var(_)
            | NirExpr::Int(_)
            | NirExpr::Bool(_)
            | NirExpr::F32(_)
            | NirExpr::F64(_) => expr.clone(),
            _ => return None,
        })
    }
}
