use super::*;
use nuis_semantics::model::NirParam;

#[cfg(test)]
#[path = "capture_params_tests.rs"]
mod tests;

// Compiler-created value helpers have a private transport, not a public ABI.
// Use only nonnegative i64 words: every encode/decode operation is total even
// under checked arithmetic, including the top (62nd) bit.
const BOOLS_PER_WORD: usize = 63;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Slot {
    Scalar(usize),
    Bools(Vec<usize>),
}

#[derive(Clone)]
pub(in crate::lowering) struct CapturePlan {
    leaves: Vec<NirParam>,
    slots: Vec<Slot>,
}

impl CapturePlan {
    pub(in crate::lowering) fn new(leaves: Vec<NirParam>) -> Option<Self> {
        let bools = leaves
            .iter()
            .enumerate()
            .filter_map(|(index, param)| (param.ty.name == "bool").then_some(index))
            .collect::<Vec<_>>();
        if bools.len() < 2 {
            return None;
        }
        let groups = bools
            .chunks(BOOLS_PER_WORD)
            .filter(|group| group.len() > 1)
            .map(|group| (group[0], group.to_vec()))
            .collect::<BTreeMap<_, _>>();
        let packed = groups.values().flatten().copied().collect::<BTreeSet<_>>();
        let slots = (0..leaves.len())
            .filter_map(|index| {
                if let Some(group) = groups.get(&index) {
                    Some(Slot::Bools(group.clone()))
                } else if !packed.contains(&index) {
                    Some(Slot::Scalar(index))
                } else {
                    None
                }
            })
            .collect();
        Some(Self { leaves, slots })
    }

    pub(super) fn lower_parameters(
        &self,
        function: &NirFunction,
        state: &mut LoweringState<'_>,
        bindings: &mut BTreeMap<String, String>,
        parameters: &mut Vec<YirFunctionParameter>,
    ) -> Result<(), String> {
        let mut decoded = BTreeMap::new();
        let mut physical_index = 0;
        for slot in &self.slots {
            let param = match slot {
                Slot::Scalar(index) => self.leaves[*index].clone(),
                Slot::Bools(group) => NirParam {
                    // The first replaced leaf is already unique in the logical
                    // signature; no synthetic name can shadow another capture.
                    name: self.leaves[group[0]].name.clone(),
                    ty: NirTypeRef {
                        name: "i64".into(),
                        generic_args: vec![],
                        is_ref: false,
                        is_optional: false,
                    },
                },
            };
            let word = aggregate_params::materialize_scalar_parameter(
                &function.name,
                &param.name,
                &param.ty,
                &mut physical_index,
                state,
                parameters,
            )?;
            match slot {
                Slot::Scalar(index) => {
                    decoded.insert(self.leaves[*index].name.clone(), word);
                }
                Slot::Bools(group) => {
                    let two = constant(2, state);
                    for (bit, index) in group.iter().enumerate() {
                        let shifted = if bit == 0 {
                            word.clone()
                        } else {
                            let divisor = constant(1_i64 << bit, state);
                            operation("div", vec![word.clone(), divisor], state)
                        };
                        let bit = operation("rem", vec![shifted, two.clone()], state);
                        let value = operation("cast_i64_to_bool", vec![bit], state);
                        decoded.insert(self.leaves[*index].name.clone(), value);
                    }
                }
            }
        }
        for param in &function.params {
            let node = restore(&param.name, &param.ty, &decoded, state)?;
            bindings.insert(param.name.clone(), node);
        }
        Ok(())
    }

    pub(super) fn lower_arguments(
        &self,
        args: &[String],
        state: &mut LoweringState<'_>,
    ) -> Result<Vec<String>, String> {
        if args.len() != self.leaves.len() {
            return Err("private value capture layout/argument mismatch".into());
        }
        let mut packed = Vec::new();
        for slot in &self.slots {
            match slot {
                Slot::Scalar(index) => packed.push(args[*index].clone()),
                Slot::Bools(group) => {
                    let mut word = None;
                    for (bit, index) in group.iter().enumerate() {
                        let value =
                            operation("cast_bool_to_i64", vec![args[*index].clone()], state);
                        let value = if bit == 0 {
                            value
                        } else {
                            let factor = constant(1_i64 << bit, state);
                            operation("mul", vec![value, factor], state)
                        };
                        word = Some(match word {
                            Some(previous) => operation("add", vec![previous, value], state),
                            None => value,
                        });
                    }
                    packed.push(word.expect("nonempty boolean capture group"));
                }
            }
        }
        Ok(packed)
    }
}

fn restore(
    path: &str,
    ty: &NirTypeRef,
    decoded: &BTreeMap<String, String>,
    state: &mut LoweringState<'_>,
) -> Result<String, String> {
    if let Some(node) = decoded.get(path) {
        return Ok(node.clone());
    }
    let fields = aggregate_params::struct_fields(ty, state)?;
    let mut args = vec![ty.name.clone()];
    let mut dependencies = Vec::new();
    for (field, ty) in fields {
        let node = restore(&format!("{path}.{field}"), &ty, decoded, state)?;
        args.push(format!("{field}={node}"));
        dependencies.push(node);
    }
    Ok(emit("struct", args, &dependencies, state))
}

fn constant(value: i64, state: &mut LoweringState<'_>) -> String {
    emit("const_i64", vec![value.to_string()], &[], state)
}

fn operation(instruction: &str, args: Vec<String>, state: &mut LoweringState<'_>) -> String {
    let dependencies = args.clone();
    emit(instruction, args, &dependencies, state)
}

fn emit(
    instruction: &str,
    args: Vec<String>,
    dependencies: &[String],
    state: &mut LoweringState<'_>,
) -> String {
    let name = next_name(state, "capture");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".into(),
        op: Operation {
            module: "cpu".into(),
            instruction: instruction.into(),
            args,
        },
    });
    for dependency in dependencies {
        push_dep_edges(state, dependency, &name);
    }
    name
}
