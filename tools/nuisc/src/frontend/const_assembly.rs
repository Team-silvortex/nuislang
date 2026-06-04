use std::collections::BTreeMap;

use nuis_semantics::model::{AstModule, AstTypeAlias, NirConstItem, NirStructDef};

use super::{
    ast_const_type_env, build_visible_type_alias_map, lower_module_const_items, FunctionSignature,
    ModuleConstValue, NirVisibility,
};

pub(super) type HelperConstMaps = BTreeMap<String, BTreeMap<String, ModuleConstValue>>;

pub(super) struct ConstAssembly {
    pub(super) lowered_consts: Vec<NirConstItem>,
    pub(super) helper_const_maps: HelperConstMaps,
    pub(super) module_const_values: BTreeMap<String, ModuleConstValue>,
    pub(super) module_const_env: BTreeMap<String, nuis_semantics::model::AstTypeRef>,
}

pub(super) fn assemble_module_consts(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<ConstAssembly, String> {
    let mut helper_const_maps = BTreeMap::<String, BTreeMap<String, ModuleConstValue>>::new();
    let mut visible_helper_consts = BTreeMap::<String, ModuleConstValue>::new();
    for helper in local_cpu_helpers {
        let helper_aliases = build_visible_type_alias_map(helper, &[])?;
        let (_, helper_consts) = lower_module_const_items(
            helper,
            &BTreeMap::new(),
            &helper_aliases,
            signatures,
            struct_table,
        )?;
        for (name, constant) in &helper_consts {
            if matches!(constant.visibility, NirVisibility::Public) {
                visible_helper_consts.insert(format!("{}.{}", helper.unit, name), constant.clone());
                visible_helper_consts
                    .entry(name.clone())
                    .or_insert_with(|| constant.clone());
            }
        }
        helper_const_maps.insert(helper.unit.clone(), helper_consts);
    }
    let (lowered_consts, module_local_consts) = lower_module_const_items(
        module,
        &visible_helper_consts,
        visible_type_aliases,
        signatures,
        struct_table,
    )?;
    let mut module_const_values = visible_helper_consts.clone();
    module_const_values.extend(module_local_consts);
    let module_const_env = ast_const_type_env(&module_const_values);

    Ok(ConstAssembly {
        lowered_consts,
        helper_const_maps,
        module_const_values,
        module_const_env,
    })
}
