use std::collections::BTreeMap;

use yir_core::Node;

use super::value_transport::NativeValueLayout;
use crate::{fresh_block, fresh_reg, LlvmValueRef, StructLlvmValueRef};

#[cfg(test)]
mod tests;

/// Lowering-private value return, never an owned runtime pointer or external ABI.
#[derive(Clone)]
pub(crate) struct NativeValueReturn {
    transport: NativeValueLayout,
}

impl NativeValueReturn {
    pub(crate) fn helper(encoded: &str) -> Result<Self, String> {
        Ok(Self {
            transport: NativeValueLayout::parse(encoded)?,
        })
    }

    pub(crate) fn callback(encoded: &str) -> Result<Self, String> {
        Self::helper(encoded)
    }

    pub(crate) fn llvm_type(&self) -> String {
        self.transport.llvm_type()
    }

    pub(crate) fn unpack(
        &self,
        returned: &str,
        body: &mut Vec<String>,
        next_reg: &mut usize,
    ) -> StructLlvmValueRef {
        self.transport.unpack(returned, body, next_reg)
    }

    /// Some(true) terminates the body; Some(false) emits a guarded early return.
    pub(crate) fn lower(
        &self,
        node: &Node,
        registers: &BTreeMap<String, LlvmValueRef>,
        body: &mut Vec<String>,
        next_reg: &mut usize,
        next_block: &mut usize,
    ) -> Result<Option<bool>, String> {
        let guarded = match node.op.instruction.as_str() {
            "return_owned_struct" => false,
            "guard_return" => true,
            _ => return Ok(None),
        };
        let value_index = usize::from(guarded);
        if let Some(encoded) = node.op.args.get(value_index + 1) {
            if &yir_core::parse_owned_struct_layout(encoded)? != self.transport.layout() {
                return Err(format!(
                    "native value return `{}` changes its function return layout",
                    node.name
                ));
            }
        }
        let value = node
            .op
            .args
            .get(value_index)
            .and_then(|name| registers.get(name))
            .ok_or_else(|| format!("native value return `{}` has no value", node.name))?;
        let value = self.transport.prepare(value).map_err(|_| {
            format!(
                "native value return `{}` does not match its function return layout",
                node.name
            )
        })?;
        let continuation = if guarded {
            let condition = match registers.get(&node.op.args[0]) {
                Some(LlvmValueRef::Bool { i1, .. }) => i1.clone(),
                Some(LlvmValueRef::I64(value)) => {
                    let condition = fresh_reg(next_reg);
                    body.push(format!("  {condition} = icmp ne i64 {value}, 0"));
                    condition
                }
                _ => return Err("native value guard requires bool or i64".to_owned()),
            };
            let selected = fresh_block(next_block, "guard_return_struct_then");
            let continuation = fresh_block(next_block, "guard_return_struct_cont");
            body.push(format!(
                "  br i1 {condition}, label %{selected}, label %{continuation}"
            ));
            body.push(format!("{selected}:"));
            Some(continuation)
        } else {
            None
        };
        let ty = self.llvm_type();
        let aggregate = value.emit(body, next_reg);
        body.push(format!("  ret {ty} {aggregate}"));
        if let Some(continuation) = continuation {
            body.push(format!("{continuation}:"));
        }
        Ok(Some(!guarded))
    }
}
