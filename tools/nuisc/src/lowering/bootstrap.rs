use super::*;
use crate::lowering::direct_calls::collect_recursive_direct_call_functions;

pub(super) trait BootstrapLoweringProvider {
    fn lowering_entry(&self) -> &'static str;
    fn lower(&self, module: &NirModule) -> Result<YirModule, String>;
}

pub(super) fn dispatch_nustar_lowering(
    module: &NirModule,
    nustar_manifest: &NustarPackageManifest,
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
    provider.lower(module)
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

    fn lower(&self, module: &NirModule) -> Result<YirModule, String> {
        lower_nir_to_yir_builtin_cpu(module)
    }
}

pub(super) fn lower_nir_to_yir_builtin_cpu(module: &NirModule) -> Result<YirModule, String> {
    if module.domain != "cpu" {
        return Err(format!(
            "minimal nuisc lowering currently only supports `mod cpu`, found `{}`",
            module.domain
        ));
    }

    let rewritten_module = rewrite_self_tail_recursive_functions(module);
    let module = &rewritten_module;
    let direct_call_functions = collect_recursive_direct_call_functions(module);

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
    yir.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.arm64"),
    });

    let mut state = LoweringState {
        yir: &mut yir,
        function_map,
        direct_call_functions: direct_call_functions.clone(),
        pure_helpers: collect_pure_helper_functions(module),
        value_counter: 0,
        print_counter: 0,
        await_counter: 0,
        call_stack: Vec::new(),
        last_effect_anchor: None,
    };

    for function in module
        .functions
        .iter()
        .filter(|function| direct_call_functions.contains(&function.name))
    {
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
        lower_function_body(main, &mut state, &mut bindings, true)?;
    }
    assign_default_lanes(&mut yir);
    materialize_registered_scheduler_contract_nodes(&mut yir);
    assign_default_lanes(&mut yir);

    Ok(yir)
}
