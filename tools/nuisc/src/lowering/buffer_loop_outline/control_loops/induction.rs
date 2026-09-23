use super::*;

pub(in crate::lowering::buffer_loop_outline) struct Iteration<'a> {
    pub prepared: PreparedCountedWhile,
    pub step: &'a NirStmt,
    pub effects: &'a [NirStmt],
    pub leading: bool,
}

pub(in crate::lowering::buffer_loop_outline) fn parse<'a>(
    condition: &NirExpr,
    body: &'a [NirStmt],
) -> Option<Iteration<'a>> {
    let prepare = |step| {
        prepare_counted_while(
            condition,
            std::slice::from_ref(step),
            &BTreeSet::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
    };
    let (first, tail) = body.split_first()?;
    if let Some(prepared) = prepare(first) {
        return Some(Iteration {
            prepared,
            step: first,
            effects: tail,
            leading: true,
        });
    }
    let (last, prefix) = body.split_last()?;
    Some(Iteration {
        prepared: prepare(last)?,
        step: last,
        effects: prefix,
        leading: false,
    })
}

impl Iteration<'_> {
    pub(in crate::lowering::buffer_loop_outline) fn normalize(
        &self,
        scope: &Scope,
    ) -> Option<Option<control_flow::Normalized>> {
        if self.leading {
            control_flow::normalize_leading(self.effects, scope)
        } else {
            control_flow::normalize_trailing(self.effects, scope, self.step)
        }
    }
}
