use super::*;
use crate::lowering::direct_calls::collect_async_loop_step_functions;
use crate::lowering::direct_calls::collect_recursive_async_helper_functions;
use crate::lowering::direct_calls::collect_recursive_direct_call_functions;

pub(super) trait BootstrapLoweringProvider {
    fn lowering_entry(&self) -> &'static str;
    fn lower(
        &self,
        module: &NirModule,
        target_config: Option<&LoweringTargetConfig>,
    ) -> Result<YirModule, String>;
}

pub(super) fn dispatch_nustar_lowering(
    module: &NirModule,
    nustar_manifest: &NustarPackageManifest,
    target_config: Option<&LoweringTargetConfig>,
) -> Result<YirModule, String> {
    if nustar_manifest.domain_family != module.domain {
        return Err(format!(
            "nustar package `{}` cannot lower mod domain `{}`",
            nustar_manifest.package_id, module.domain
        ));
    }
    let provider = bootstrap_lowering_provider(nustar_manifest.yir_lowering_entry.as_str())
        .ok_or_else(|| {
            format!(
                "nuisc scheduler has no bootstrap compatibility shim for lowering entry `{}`; this must be provided by the loaded nustar implementation",
                nustar_manifest.yir_lowering_entry
            )
        })?;
    validate_lowering_target(module, nustar_manifest, target_config)?;
    provider.lower(module, target_config)
}

fn bootstrap_lowering_provider(entry: &str) -> Option<&'static dyn BootstrapLoweringProvider> {
    static CPU_PROVIDER: CpuBootstrapLoweringProvider = CpuBootstrapLoweringProvider;
    [(&CPU_PROVIDER as &dyn BootstrapLoweringProvider)]
        .into_iter()
        .find(|provider| provider.lowering_entry() == entry)
}

struct CpuBootstrapLoweringProvider;

impl BootstrapLoweringProvider for CpuBootstrapLoweringProvider {
    fn lowering_entry(&self) -> &'static str {
        "cpu.yir.lowering.v1"
    }

    fn lower(
        &self,
        module: &NirModule,
        target_config: Option<&LoweringTargetConfig>,
    ) -> Result<YirModule, String> {
        lower_nir_to_yir_builtin_cpu_with_target(module, target_config)
    }
}

#[cfg(test)]
pub(super) fn lower_nir_to_yir_builtin_cpu(module: &NirModule) -> Result<YirModule, String> {
    lower_nir_to_yir_builtin_cpu_with_target(module, None)
}

pub(super) fn lower_nir_to_yir_builtin_cpu_with_target(
    module: &NirModule,
    target_config: Option<&LoweringTargetConfig>,
) -> Result<YirModule, String> {
    if module.domain != "cpu" {
        return Err(format!(
            "minimal nuisc lowering currently only supports `mod cpu`, found `{}`",
            module.domain
        ));
    }

    let rewritten_module = rewrite_self_tail_recursive_functions(module);
    let module = &rewritten_module;
    let direct_call_functions = collect_recursive_direct_call_functions(module);
    let async_helper_functions = collect_recursive_async_helper_functions(module);
    let async_loop_step_functions = collect_async_loop_step_functions(module);

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .ok_or_else(|| "minimal nuisc lowering expects `fn main()`".to_owned())?;

    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();

    let mut yir = YirModule::new("0.1");
    let cpu_resource_kind = target_config
        .map(|target| format!("cpu.{}", target.machine_arch))
        .unwrap_or_else(|| "cpu.arm64".to_owned());
    yir.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse(&cpu_resource_kind),
    });

    let mut state = LoweringState {
        yir: &mut yir,
        function_map,
        direct_call_functions: direct_call_functions.clone(),
        async_helper_functions: async_helper_functions
            .union(&async_loop_step_functions)
            .cloned()
            .collect(),
        pure_helpers: collect_pure_helper_functions(module),
        inlineable_pure_helpers: collect_inlineable_pure_helper_exprs(module),
        pure_helper_blocks: collect_pure_helper_blocks(module),
        value_counter: 0,
        print_counter: 0,
        await_counter: 0,
        call_stack: Vec::new(),
        last_effect_anchor: None,
        target_config: target_config.cloned(),
    };

    materialize_cpu_target_config_node(&mut state);

    for function in module.functions.iter().filter(|function| {
        direct_call_functions.contains(&function.name)
            || async_helper_functions.contains(&function.name)
            || async_loop_step_functions.contains(&function.name)
    }) {
        lower_direct_call_helper_function(function, &mut state)?;
    }

    if direct_call_functions.contains("main") {
        let entry = push_direct_call_node(main, &[], &mut state)?;
        let entry_return = next_name(&mut state, "entry_return");
        state.yir.nodes.push(Node {
            name: entry_return.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "return_i64".to_owned(),
                args: vec![entry.clone()],
            },
        });
        push_dep_edges(&mut state, &entry, &entry_return);
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: entry,
            to: entry_return,
        });
    } else {
        let mut bindings = BTreeMap::<String, String>::new();
        let returned = lower_function_body(main, &mut state, &mut bindings, true)?;
        if returned.is_none() && main.return_type.is_none() {
            let value = next_name(&mut state, "implicit_main_return_value");
            state.yir.nodes.push(Node {
                name: value.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "const_i64".to_owned(),
                    args: vec!["0".to_owned()],
                },
            });
            let name = next_name(&mut state, "implicit_main_return");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "return_i64".to_owned(),
                    args: vec![value.clone()],
                },
            });
            push_dep_edges(&mut state, &value, &name);
        }
    }
    materialize_doc_contract_nodes(&mut yir, module);
    assign_default_lanes(&mut yir);
    materialize_registered_scheduler_contract_nodes(&mut yir);
    assign_default_lanes(&mut yir);

    Ok(yir)
}

fn validate_lowering_target(
    module: &NirModule,
    manifest: &NustarPackageManifest,
    target_config: Option<&LoweringTargetConfig>,
) -> Result<(), String> {
    let Some(target_config) = target_config else {
        return Ok(());
    };
    crate::registry::validate_manifest_abi(manifest, &target_config.abi)?;
    if module.domain == "cpu" {
        let registered = crate::registry::registered_abi_target(manifest, &target_config.abi)?;
        if registered.machine_arch != target_config.machine_arch
            || registered.machine_os != target_config.machine_os
            || registered.object_format != target_config.object_format
            || registered.calling_abi != target_config.calling_abi
            || registered.clang_target != target_config.clang_target
        {
            return Err(format!(
                "lowering target `{}` does not match registered ABI target metadata for domain `{}`",
                target_config.abi, module.domain
            ));
        }
    }
    Ok(())
}

fn materialize_cpu_target_config_node(state: &mut LoweringState<'_>) {
    let Some(target_config) = state.target_config.clone() else {
        return;
    };
    let arch = target_config.machine_arch.clone();
    let abi = target_config.abi.clone();
    let vector_bits = target_config.cpu_vector_bits().to_string();
    let isa_family = target_config.isa_family.clone();
    let isa_features = target_config.isa_features.join(",");
    let name = "lowering_cpu_target_config".to_owned();
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "target_config".to_owned(),
            args: vec![
                arch.clone(),
                abi.clone(),
                vector_bits.clone(),
                isa_family.clone(),
                isa_features.clone(),
            ],
        },
    });
    let contract_name = "lowering_cpu_target_contract_type".to_owned();
    state.yir.nodes.push(Node {
        name: contract_name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "text".to_owned(),
            args: vec![format!(
                "arch=symbol:{};abi=symbol:{};vector_bits=i64:{};isa_family=symbol:{};isa_features=list:{}",
                arch, abi, vector_bits, isa_family, isa_features
            )],
        },
    });
    push_dep_edges(state, &contract_name, &name);
}
