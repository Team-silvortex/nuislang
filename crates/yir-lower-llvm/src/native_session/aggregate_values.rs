use std::collections::BTreeMap;

use yir_core::{Node, OwnedStructLayout};

use crate::{fresh_block, fresh_reg, LlvmValueRef, StructLlvmValueRef};

#[cfg(test)]
mod tests;

/// Lowering-private value return, never an owned runtime pointer or external ABI.
#[derive(Clone)]
pub(crate) struct NativeValueReturn {
    layout: OwnedStructLayout,
    slots: usize,
}

impl NativeValueReturn {
    pub(crate) fn flat_i64(layout: &OwnedStructLayout) -> Result<Self, String> {
        if !super::aggregates::flat_i64_values(layout) {
            return Err("native value return requires a bounded flat i64 layout".to_owned());
        }
        Ok(Self {
            layout: layout.clone(),
            slots: layout.fields.len(),
        })
    }

    pub(crate) fn callback(encoded: &str) -> Result<Self, String> {
        let state = super::ScalarStateLayout::parse(encoded)?;
        Ok(Self {
            layout: yir_core::parse_owned_struct_layout(encoded)?,
            slots: state.fields().len(),
        })
    }

    pub(crate) fn llvm_type(&self) -> String {
        format!("[{} x i64]", self.slots)
    }

    pub(crate) fn unpack(
        &self,
        returned: &str,
        body: &mut Vec<String>,
        next_reg: &mut usize,
    ) -> StructLlvmValueRef {
        let template = crate::call_lowering::owned_struct_layout_template(self.layout.clone());
        let LlvmValueRef::Struct(value) = unpack(
            &LlvmValueRef::Struct(template),
            returned,
            &self.llvm_type(),
            &mut 0,
            body,
            next_reg,
        ) else {
            unreachable!("validated native struct return")
        };
        value
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
            if yir_core::parse_owned_struct_layout(encoded)? != self.layout {
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
        let template = crate::call_lowering::owned_struct_layout_template(self.layout.clone());
        let Some(LlvmValueRef::Struct(value)) = crate::task_owned_payload::materialize_owned_value(
            value,
            &LlvmValueRef::Struct(template),
        ) else {
            return Err(format!(
                "native value return `{}` does not match its function return layout",
                node.name
            ));
        };
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
        let mut aggregate = "poison".to_owned();
        let mut words = Vec::with_capacity(self.slots);
        pack(&LlvmValueRef::Struct(value), &mut words, body, next_reg)?;
        for (index, field) in words.iter().enumerate() {
            let inserted = fresh_reg(next_reg);
            body.push(format!(
                "  {inserted} = insertvalue {ty} {aggregate}, i64 {field}, {index}"
            ));
            aggregate = inserted;
        }
        body.push(format!("  ret {ty} {aggregate}"));
        if let Some(continuation) = continuation {
            body.push(format!("{continuation}:"));
        }
        Ok(Some(!guarded))
    }
}

fn pack(
    value: &LlvmValueRef,
    words: &mut Vec<String>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Result<(), String> {
    if let LlvmValueRef::Struct(value) = value {
        for (_, field) in &value.fields {
            pack(field, words, body, next_reg)?;
        }
    } else {
        words.push(
            crate::task_owned_payload::pack_scalar(value, body, next_reg)
                .ok_or("native value return changed its scalar kind")?,
        );
    }
    Ok(())
}

fn unpack(
    template: &LlvmValueRef,
    returned: &str,
    ty: &str,
    slot: &mut usize,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> LlvmValueRef {
    if let LlvmValueRef::Struct(template) = template {
        LlvmValueRef::Struct(StructLlvmValueRef {
            type_name: template.type_name.clone(),
            fields: template
                .fields
                .iter()
                .map(|(name, field)| {
                    (
                        name.clone(),
                        unpack(field, returned, ty, slot, body, next_reg),
                    )
                })
                .collect(),
        })
    } else {
        let word = fresh_reg(next_reg);
        body.push(format!("  {word} = extractvalue {ty} {returned}, {slot}"));
        *slot += 1;
        crate::task_owned_payload::unpack_scalar(&word, template, body, next_reg)
            .expect("validated native scalar return")
    }
}
