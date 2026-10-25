use super::*;
use scalar_joins::nested_prefix::{Arm, Join, JoinKind, Local};

impl<L: control_values::ValueLayouts> Builder<'_, L> {
    pub(super) fn nested_declaration(
        &mut self,
        stmt: &mut NirStmt,
        scope: &mut Scope,
        suffix: &[NirStmt],
    ) -> bool {
        self.typed_nested_declaration(stmt, scope, suffix, JoinKind::Scalars)
    }

    pub(super) fn record_declaration(
        &mut self,
        stmt: &mut NirStmt,
        scope: &mut Scope,
        suffix: &[NirStmt],
    ) -> bool {
        self.typed_nested_declaration(stmt, scope, suffix, JoinKind::Records)
    }

    pub(super) fn discarded_record_declaration(
        &mut self,
        stmt: &mut NirStmt,
        scope: &mut Scope,
        suffix: &[NirStmt],
    ) -> bool {
        self.typed_nested_declaration(stmt, scope, suffix, JoinKind::DiscardedRecords)
    }

    pub(super) fn outer_record_declaration(
        &mut self,
        stmt: &mut NirStmt,
        scope: &mut Scope,
        suffix: &[NirStmt],
    ) -> bool {
        self.typed_nested_declaration(stmt, scope, suffix, JoinKind::OuterRecords)
    }

    pub(super) fn logical_field_declaration(
        &mut self,
        stmt: &mut NirStmt,
        scope: &mut Scope,
        suffix: &[NirStmt],
    ) -> bool {
        self.typed_nested_declaration(stmt, scope, suffix, JoinKind::LogicalFields)
    }

    fn typed_nested_declaration(
        &mut self,
        stmt: &mut NirStmt,
        scope: &mut Scope,
        suffix: &[NirStmt],
        kind: JoinKind,
    ) -> bool {
        let join = match kind {
            JoinKind::Continuation => unreachable!("continuations require a whole-function plan"),
            JoinKind::LogicalFields => scalar_joins::nested_prefix::logical_field_declaration(
                stmt,
                scope,
                suffix,
                self.catalog,
                self.layouts,
            ),
            JoinKind::OuterRecords => scalar_joins::nested_prefix::outer_record_declaration(
                stmt,
                scope,
                suffix,
                self.catalog,
                self.layouts,
            ),
            JoinKind::Scalars => scalar_joins::nested_prefix::declaration(
                stmt,
                scope,
                suffix,
                self.catalog,
                self.layouts,
            ),
            JoinKind::Records => scalar_joins::nested_prefix::record_declaration(
                stmt,
                scope,
                suffix,
                self.catalog,
                self.layouts,
            ),
            JoinKind::DiscardedRecords => {
                scalar_joins::nested_prefix::discarded_record_declaration(
                    stmt,
                    scope,
                    suffix,
                    self.catalog,
                    self.layouts,
                )
            }
        };
        let Some(join) = join else {
            return false;
        };
        let fields = kind == JoinKind::LogicalFields;
        *stmt = self.joined_declaration(&join, scope, fields);
        true
    }

    pub(super) fn joined_declaration(
        &mut self,
        join: &Join<'_>,
        scope: &mut Scope,
        fields: bool,
    ) -> NirStmt {
        let name = join.yes.name.to_owned();
        let ty = join.yes.ty.clone();
        let yes = self.nested_arm(&join.yes, scope, fields);
        let no = self.nested_arm(&join.no, scope, fields);
        let condition = self
            .gated_condition(join.condition, scope)
            .unwrap_or_else(|| join.condition.clone());
        let stmt = self.extract(condition, yes, no, scope, false);
        scope.insert(name, ty);
        stmt
    }

    fn nested_arm(&mut self, arm: &Arm<'_>, scope: &Scope, fields: bool) -> Selection {
        let mut inner = scope.clone();
        let mut locals = Vec::new();
        for local in &arm.locals {
            match local {
                Local::Binding { stmt, ty } => {
                    let (NirStmt::Let { name, .. } | NirStmt::Const { name, .. }) = stmt else {
                        unreachable!()
                    };
                    let mut stmt = (*stmt).clone();
                    if let NirStmt::Let { ty: declared, .. } = &mut stmt {
                        *declared = Some(ty.clone());
                    }
                    if fields {
                        let (NirStmt::Let { value, .. } | NirStmt::Const { value, .. }) = &mut stmt
                        else {
                            unreachable!()
                        };
                        *value = self.record_field_value(value, &inner);
                    }
                    inner.insert(name.clone(), ty.clone());
                    locals.push(stmt);
                }
                Local::Join { join, export } => {
                    let yes = self.nested_arm(&join.yes, &inner, fields);
                    let no = self.nested_arm(&join.no, &inner, fields);
                    let condition = self
                        .gated_condition(join.condition, &inner)
                        .unwrap_or_else(|| join.condition.clone());
                    locals.push(self.extract(condition, yes, no, &inner, !*export));
                    if *export {
                        inner.insert(join.yes.name.to_owned(), join.yes.ty.clone());
                    }
                }
                Local::Condition(value) => {
                    locals.push(NirStmt::Let {
                        name: branches::fresh_name(
                            "__nuis_discarded_condition",
                            &mut self.bindings,
                        ),
                        ty: Some(scalar_type("bool")),
                        value: (*value).clone(),
                    });
                }
            }
        }
        let selected = Selection {
            name: arm.name.to_owned(),
            ty: arm.ty.clone(),
            constant: false,
            body: vec![NirStmt::Return(Some(if fields {
                self.record_field_value(arm.value.as_ref(), &inner)
            } else {
                arm.value.as_ref().clone()
            }))],
            inputs: arm.inputs.clone(),
        }
        .with_prefix(locals);
        // Explicit original types keep generated calls out of old-catalog inference.
        self.guard_selection(selected, scope, false)
    }
}
