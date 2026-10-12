use super::*;

// Pair authority stays separate from the field-type-driven record assembler.
#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    plan: Plan,
    owner: String,
    child: String,
    field_types: (NirTypeRef, NirTypeRef),
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    structs: &mut Vec<NirStructDef>,
) -> Vec<NirStmt> {
    let fields = [("owner", field_types.0), ("child", field_types.1)];
    super::record::install(
        plan,
        fields
            .into_iter()
            .zip([owner, child])
            .map(|((label, ty), name)| (label.into(), name, ty))
            .collect(),
        "__nuis_effect_pair_value",
        "__nuis_effect_call_pair",
        names,
        bindings,
        helpers,
        structs,
    )
}
