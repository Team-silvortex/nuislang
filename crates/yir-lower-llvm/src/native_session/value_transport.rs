use yir_core::{OwnedStructFieldLayout, OwnedStructLayout, OwnedStructScalarLayout};

use crate::{fresh_reg, LlvmValueRef, StructLlvmValueRef};

#[cfg(test)]
mod tests;

/// A bounded pure-value codec, not authority to change a function or external ABI.
#[derive(Clone)]
pub(crate) struct NativeValueLayout {
    layout: OwnedStructLayout,
    slots: usize,
}

/// Only a checked layout can create this ordered, resource-free scalar snapshot.
pub(crate) struct PreparedNativeValue {
    words: Vec<LlvmValueRef>,
}

impl NativeValueLayout {
    pub(crate) fn parse(encoded: &str) -> Result<Self, String> {
        let (layout, slots) = super::aggregates::scalar_value_layout(encoded)?;
        Ok(Self { layout, slots })
    }

    pub(crate) fn layout(&self) -> &OwnedStructLayout {
        &self.layout
    }

    pub(crate) fn llvm_type(&self) -> String {
        format!("[{} x i64]", self.slots)
    }

    /// Validate the entire nominal tree before emitting instructions or labels.
    pub(crate) fn prepare(&self, value: &LlvmValueRef) -> Result<PreparedNativeValue, String> {
        let mut words = Vec::with_capacity(self.slots);
        prepare_record(&self.layout, value, &mut words)
            .ok_or("native value does not match its declared transport layout")?;
        debug_assert_eq!(words.len(), self.slots);
        Ok(PreparedNativeValue { words })
    }

    pub(crate) fn unpack(
        &self,
        aggregate: &str,
        body: &mut Vec<String>,
        next_reg: &mut usize,
    ) -> StructLlvmValueRef {
        let template = crate::call_lowering::owned_struct_layout_template(self.layout.clone());
        let LlvmValueRef::Struct(value) = unpack(
            &LlvmValueRef::Struct(template),
            aggregate,
            &self.llvm_type(),
            &mut 0,
            body,
            next_reg,
        ) else {
            unreachable!("validated native value layout")
        };
        value
    }
}

impl PreparedNativeValue {
    pub(crate) fn emit(&self, body: &mut Vec<String>, next_reg: &mut usize) -> String {
        let words = self
            .words
            .iter()
            .map(|value| {
                crate::task_owned_payload::pack_scalar(value, body, next_reg)
                    .expect("prepared native values contain only scalar leaves")
            })
            .collect::<Vec<_>>();
        let ty = format!("[{} x i64]", words.len());
        let mut aggregate = "poison".to_owned();
        for (index, field) in words.iter().enumerate() {
            let inserted = fresh_reg(next_reg);
            body.push(format!(
                "  {inserted} = insertvalue {ty} {aggregate}, i64 {field}, {index}"
            ));
            aggregate = inserted;
        }
        aggregate
    }
}

fn prepare_record(
    layout: &OwnedStructLayout,
    value: &LlvmValueRef,
    words: &mut Vec<LlvmValueRef>,
) -> Option<()> {
    let LlvmValueRef::Struct(value) = value else {
        return None;
    };
    // Pure records must not inherit owned-variant conversion or zero-fill rules.
    if value.type_name != layout.type_name || value.fields.len() != layout.fields.len() {
        return None;
    }
    for (name, field) in &layout.fields {
        let (_, value) = value
            .fields
            .iter()
            .find(|(candidate, _)| candidate == name)?;
        match field {
            OwnedStructFieldLayout::Struct(layout) => prepare_record(layout, value, words)?,
            OwnedStructFieldLayout::Scalar(kind) => {
                if !matches!(
                    (kind, value),
                    (OwnedStructScalarLayout::Bool, LlvmValueRef::Bool { .. })
                        | (OwnedStructScalarLayout::I32, LlvmValueRef::I32(_))
                        | (OwnedStructScalarLayout::I64, LlvmValueRef::I64(_))
                        | (OwnedStructScalarLayout::F32, LlvmValueRef::F32(_))
                        | (OwnedStructScalarLayout::F64, LlvmValueRef::F64(_))
                ) {
                    return None;
                }
                words.push(value.clone());
            }
        }
    }
    Some(())
}

fn unpack(
    template: &LlvmValueRef,
    aggregate: &str,
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
                        unpack(field, aggregate, ty, slot, body, next_reg),
                    )
                })
                .collect(),
        })
    } else {
        let word = fresh_reg(next_reg);
        body.push(format!("  {word} = extractvalue {ty} {aggregate}, {slot}"));
        *slot += 1;
        crate::task_owned_payload::unpack_scalar(&word, template, body, next_reg)
            .expect("validated native scalar transport")
    }
}
