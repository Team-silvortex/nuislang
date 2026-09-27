use crate::{native_session::value_transport::NativeValueLayout, CpuCallScalarKind, LlvmValueRef};
use yir_core::Node;

#[derive(Clone)]
pub(crate) enum CpuCallParameterKind {
    Scalar(CpuCallScalarKind),
    Record(NativeValueLayout),
}

impl CpuCallParameterKind {
    pub(crate) fn scalar(&self) -> Option<CpuCallScalarKind> {
        match self {
            Self::Scalar(kind) => Some(*kind),
            Self::Record(_) => None,
        }
    }

    pub(crate) fn llvm_type(&self) -> String {
        match self {
            Self::Scalar(kind) => crate::cpu_scalar_kind_llvm_type(*kind).to_owned(),
            Self::Record(layout) => layout.llvm_type(),
        }
    }

    pub(crate) fn validate(&self, value: &LlvmValueRef) -> Result<(), String> {
        match self {
            Self::Record(layout) => layout.prepare(value).map(|_| ()),
            Self::Scalar(kind) => crate::call_lowering::lower_scalar_value_arg(value, kind)
                .map(|_| ())
                .ok_or_else(|| {
                    "argument does not exactly match its declared scalar kind".to_owned()
                }),
        }
    }
}

impl PartialEq<CpuCallScalarKind> for CpuCallParameterKind {
    fn eq(&self, other: &CpuCallScalarKind) -> bool {
        self.scalar() == Some(*other)
    }
}

pub(crate) fn collect(
    nodes: &[&Node],
) -> Result<Vec<(usize, String, CpuCallParameterKind)>, String> {
    let mut parameters = Vec::new();
    for node in nodes {
        let (index, kind) = if let Some(parameter) = yir_domain_cpu::value_parameters::parse(node)?
        {
            (
                parameter.index,
                CpuCallParameterKind::Record(NativeValueLayout::parse(&node.op.args[1])?),
            )
        } else {
            if !node.op.instruction.starts_with("param_") {
                continue;
            }
            let Some(kind) = crate::cpu_call_scalar_kind_for_instruction(&node.op.instruction)
            else {
                continue;
            };
            let index = node
                .op
                .args
                .first()
                .and_then(|arg| arg.parse().ok())
                .ok_or_else(|| format!("invalid {} parameter index", node.op.full_name()))?;
            (index, CpuCallParameterKind::Scalar(kind))
        };
        parameters.push((index, node.name.clone(), kind));
    }
    parameters.sort_by_key(|(index, _, _)| *index);
    Ok(parameters)
}
