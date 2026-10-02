use super::*;

type Live = BTreeMap<String, BTreeSet<Vec<String>>>;

#[derive(Clone, Copy)]
struct LoopTargets<'a> {
    on_break: &'a Live,
    on_continue: &'a Live,
}

// Analyze a hygienic copy only. Scoped control identities and every original
// assignment stay intact; only the proven input reconstruction may change.
pub(super) fn reconstruction(
    function: &NirFunction,
    position: usize,
    name: &str,
    input: &WordInput,
    definitions: &BTreeMap<&str, &NirStructDef>,
    layouts: &impl ValueLayouts,
) -> Option<NirExpr> {
    let mut probe = function.clone();
    bindings::normalize(&mut probe, &BTreeSet::new());
    let mut demand = Demand::new(&probe, definitions, layouts)?;
    let mut live = demand.block(&probe.body[position + 1..], Live::new(), None, 0)?;
    let needed = live.remove(name).unwrap_or_default();
    let leaves = input.shape.leaves();
    if needed.len() >= leaves.len() {
        return None;
    }
    let mut slot = 0;
    Some(input.shape.reconstruct(&mut |ty| {
        let index = slot;
        slot += 1;
        if needed.contains(&leaves[index].0) {
            scalar_carries::decode(
                ty,
                source_value(&input.param.name, &[format!("carry{index}")]),
            )
        } else {
            control_values::zero_value(ty, layouts)
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materialized_record_demand_bounds_work_and_depth_without_partial_authority() {
        let module = crate::frontend::parse_nuis_module(
            "mod cpu Main { struct State { used: f64, unused: i64 }
                fn read(input: State) -> f64 { return input.used; } }",
        )
        .unwrap();
        let definitions = module
            .structs
            .iter()
            .map(|d| (d.name.as_str(), d))
            .collect();
        let layouts = control_values::TypedLayouts::collect(&module);
        let function = &module.functions[0];
        let mut demand = Demand::new(function, &definitions, &layouts).unwrap();
        let live = demand.block(&function.body, Live::new(), None, 0).unwrap();
        assert_eq!(live["input"], BTreeSet::from([vec!["used".into()]]));
        demand.remaining = 0;
        assert!(demand.block(&function.body, Live::new(), None, 0).is_none());
        demand.remaining = 65_536;
        assert!(demand
            .block(&function.body, Live::new(), None, 64)
            .is_none());
    }

    #[test]
    fn materialized_record_demand_bounds_nested_fixed_point_work() {
        let module = crate::frontend::parse_nuis_module(
            "mod cpu Main { struct State { used: i64, flag: bool, unused: i64 }
                fn read(input: State) -> i64 {
                    let current = input;
                    while current.flag {
                        let current = current; let before = current;
                        while before.flag { let current = before; break; }
                        let current = State { used: current.used + 1, flag: false, unused: 0 };
                    }
                    return current.used;
                } }",
        )
        .unwrap();
        let definitions = module
            .structs
            .iter()
            .map(|d| (d.name.as_str(), d))
            .collect();
        let layouts = control_values::TypedLayouts::collect(&module);
        let mut probe = module.functions[0].clone();
        bindings::normalize(&mut probe, &BTreeSet::new());
        let mut demand = Demand::new(&probe, &definitions, &layouts).unwrap();
        let live = demand.block(&probe.body, Live::new(), None, 0).unwrap();
        assert_eq!(
            live["input"],
            BTreeSet::from([vec!["used".into()], vec!["flag".into()]])
        );
        demand.remaining = 8;
        assert!(demand.block(&probe.body, Live::new(), None, 0).is_none());
    }
}

struct Demand<'a, L> {
    types: Scope,
    paths: BTreeMap<String, Vec<Vec<String>>>,
    layouts: &'a L,
    remaining: usize,
}

impl<'a, L: ValueLayouts> Demand<'a, L> {
    fn new(
        function: &NirFunction,
        definitions: &BTreeMap<&str, &NirStructDef>,
        layouts: &'a L,
    ) -> Option<Self> {
        let mut types = Scope::new();
        for param in &function.params {
            if types.insert(param.name.clone(), param.ty.clone()).is_some() {
                return None;
            }
        }
        for scope in scopes::collect(&function.body) {
            for stmt in scope.body {
                let (name, ty) = match stmt {
                    NirStmt::Let { name, ty, .. } => (name, ty.as_ref()?),
                    NirStmt::Const { name, ty, .. } => (name, ty),
                    _ => continue,
                };
                if types
                    .insert(name.clone(), ty.clone())
                    .is_some_and(|old| old != *ty)
                {
                    return None;
                }
            }
        }
        let mut paths = BTreeMap::new();
        for ty in types.values() {
            if !control_values::supported_type(ty, layouts) {
                return None;
            }
            if !layouts.scalar(&ty.name) && !paths.contains_key(&ty.name) {
                paths.insert(
                    ty.name.clone(),
                    Shape::from_definitions(ty, definitions)?
                        .leaves()
                        .into_iter()
                        .map(|(path, _)| path)
                        .collect(),
                );
            }
        }
        Some(Self {
            types,
            paths,
            layouts,
            remaining: 65_536,
        })
    }

    fn charge(&mut self, amount: usize) -> Option<()> {
        self.remaining = self.remaining.checked_sub(amount)?;
        Some(())
    }

    fn block(
        &mut self,
        body: &[NirStmt],
        mut live: Live,
        targets: Option<LoopTargets<'_>>,
        depth: usize,
    ) -> Option<Live> {
        if depth >= 64 {
            return None;
        }
        for stmt in body.iter().rev() {
            self.charge(1)?;
            match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    let needed = live.remove(name).unwrap_or_default();
                    let ty = self.types.get(name)?;
                    if !self.layouts.scalar(&ty.name) {
                        if let Some(path) = access(value) {
                            let source = self.types.get(&path[0])?;
                            if field_type(source, &path[1..], self.layouts)? != *ty {
                                return None;
                            }
                            self.charge(needed.len())?;
                            for leaf in needed {
                                let mut selected = path[1..].to_vec();
                                selected.extend(leaf);
                                live.entry(path[0].clone()).or_default().insert(selected);
                            }
                            continue;
                        }
                    }
                    // Constructors/calls still execute, including unused checked
                    // fields. Observe all operands, not just the final value demand.
                    self.observe(value, &mut live)?;
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    let left = self.block(then_body, live.clone(), targets, depth + 1)?;
                    live = self.block(else_body, live, targets, depth + 1)?;
                    self.merge(&mut live, left)?;
                    self.observe(condition, &mut live)?;
                }
                NirStmt::Return(value) => {
                    live.clear();
                    if let Some(value) = value {
                        self.observe(value, &mut live)?;
                    }
                }
                NirStmt::Print(value) | NirStmt::Expr(value) | NirStmt::Await(value) => {
                    self.observe(value, &mut live)?;
                }
                NirStmt::While { condition, body } => {
                    live = self.loop_entry(condition, body, live, depth)?;
                }
                NirStmt::Break => live = targets?.on_break.clone(),
                NirStmt::Continue => live = targets?.on_continue.clone(),
            }
        }
        Some(live)
    }

    fn merge(&mut self, live: &mut Live, incoming: Live) -> Option<()> {
        for (name, paths) in incoming {
            self.charge(paths.len())?;
            live.entry(name).or_default().extend(paths);
        }
        Some(())
    }

    fn loop_entry(
        &mut self,
        condition: &NirExpr,
        body: &[NirStmt],
        after: Live,
        depth: usize,
    ) -> Option<Live> {
        // The header also has a zero-trip/normal-exit edge. Grow demand until
        // another backedge adds nothing; exhaustion abandons this candidate.
        let mut header = after.clone();
        self.observe(condition, &mut header)?;
        loop {
            self.charge(1)?;
            let targets = LoopTargets {
                on_break: &after,
                on_continue: &header,
            };
            let before = self.block(body, header.clone(), Some(targets), depth + 1)?;
            let mut next = header.clone();
            self.merge(&mut next, before)?;
            if next == header {
                return Some(header);
            }
            header = next;
        }
    }

    fn observe(&mut self, expr: &NirExpr, live: &mut Live) -> Option<()> {
        let mut pending = vec![expr];
        while let Some(expr) = pending.pop() {
            self.charge(1)?;
            if let Some(path) = access(expr) {
                let ty = self.types.get(&path[0])?;
                field_type(ty, &path[1..], self.layouts)?;
                if let Some(paths) = self.paths.get(&ty.name) {
                    self.remaining = self.remaining.checked_sub(paths.len())?;
                    for leaf in paths.iter().filter(|leaf| leaf.starts_with(&path[1..])) {
                        live.entry(path[0].clone())
                            .or_default()
                            .insert(leaf.clone());
                    }
                }
                continue;
            }
            crate::nir_walk::walk_child_exprs(expr, &mut |child| pending.push(child));
        }
        Some(())
    }
}
