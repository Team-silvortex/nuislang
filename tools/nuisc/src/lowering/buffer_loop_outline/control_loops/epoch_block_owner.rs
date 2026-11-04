use super::*;

// Entry authority belongs to this complete effects block and fresh epoch only.
// Ordinary branch recursion uses block(), which has no owner to inherit.
pub(super) struct Owner<'a> {
    body: &'a [NirStmt],
    epoch: &'a print_epoch::Epoch,
}

impl Owner<'_> {
    pub(super) fn matches(&self, body: &[NirStmt], epoch: &print_epoch::Epoch) -> bool {
        std::ptr::eq(self.body, body) && std::ptr::eq(self.epoch, epoch)
    }
}

pub(super) fn validate(
    body: &[NirStmt],
    locals: &mut Locals,
    updates: &BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &control_values::CarryLayouts,
    literal_prints: validation::LiteralPrints,
    epoch: &print_epoch::Epoch,
    depth: usize,
) -> Option<()> {
    let owner = Owner { body, epoch };
    block_with_owner(
        body,
        locals,
        updates,
        catalog,
        layouts,
        literal_prints,
        epoch,
        Some(&owner),
        depth,
    )
}

pub(super) fn validate_entry(
    body: &[NirStmt],
    normalized: Option<&control_flow::Normalized>,
    locals: &mut Locals,
    updates: &BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &control_values::CarryLayouts,
    literal_prints: validation::LiteralPrints,
    epoch: &print_epoch::Epoch,
    depth: usize,
) -> Option<()> {
    let effects = if let Some(flow) = normalized {
        let (prologue, effects) = flow.entry_blocks(body)?;
        block(
            prologue,
            locals,
            updates,
            catalog,
            layouts,
            literal_prints,
            epoch,
            depth,
        )?;
        effects
    } else {
        body
    };
    validate(
        effects,
        locals,
        updates,
        catalog,
        layouts,
        literal_prints,
        epoch,
        depth,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_local_owner_cannot_move_to_another_block_or_epoch() {
        let body = vec![NirStmt::Print(NirExpr::Int(71))];
        let epoch = print_epoch::Epoch::default();
        let owner = Owner {
            body: &body,
            epoch: &epoch,
        };
        assert!(owner.matches(&body, &epoch));
        assert!(!owner.matches(&body.clone(), &epoch));
        assert!(!owner.matches(&body[..0], &epoch));
        assert!(!owner.matches(&body, &print_epoch::Epoch::default()));
    }
}
