//! Scalar transport shared by static code producers and session hosts.
//! This contract describes values, not a compiler backend or native-code trust.

use crate::{
    ApplicationSessionSignature, OwnedStructFieldLayout, OwnedStructLayout,
    OwnedStructScalarLayout, StructValue, Value, YirModule,
};

pub const CONTRACT: &str = "nuis-native-scalar-session-bridge-v1";
pub const MAX_SCALAR_SLOTS: usize = 64;
pub const MAX_BINDING_SOURCE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarKind {
    Bool,
    I32,
    I64,
    F32,
    F64,
}

impl ScalarKind {
    pub fn parse(ty: &str) -> Result<Self, String> {
        match ty {
            "bool" => Ok(Self::Bool),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "f32" => Ok(Self::F32),
            "f64" => Ok(Self::F64),
            _ => Err(format!("native scalar bridge does not admit `{ty}`")),
        }
    }

    pub fn pack(self, value: &Value) -> Result<u64, String> {
        match (self, value) {
            (Self::Bool, Value::Bool(v)) => Ok(u64::from(*v)),
            (Self::I32, Value::I32(v)) => Ok(*v as i64 as u64),
            (Self::I64, Value::Int(v)) => Ok(*v as u64),
            (Self::F32, Value::F32(v)) => Ok(u64::from(v.to_bits())),
            (Self::F64, Value::F64(v)) => Ok(v.to_bits()),
            _ => Err("native scalar bridge argument kind mismatch".to_owned()),
        }
    }

    pub fn unpack(self, word: u64) -> Result<Value, String> {
        match self {
            Self::Bool if word <= 1 => Ok(Value::Bool(word != 0)),
            Self::I32 if word as i32 as i64 as u64 == word => Ok(Value::I32(word as i32)),
            Self::I64 => Ok(Value::Int(word as i64)),
            Self::F32 if word <= u64::from(u32::MAX) => Ok(Value::F32(f32::from_bits(word as u32))),
            Self::F64 => Ok(Value::F64(f64::from_bits(word))),
            _ => Err("native scalar bridge received a noncanonical scalar word".to_owned()),
        }
    }
}

/// A bounded, nonempty, scalar-only owned layout. Fields retain ABI order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalarStateLayout {
    source: String,
    layout: OwnedStructLayout,
    fields: Vec<(String, ScalarKind)>,
}

impl ScalarStateLayout {
    pub fn parse(source: &str) -> Result<Self, String> {
        let layout = crate::parse_owned_struct_layout(source)?;
        let mut fields = Vec::new();
        flatten(&layout, "", &mut fields)?;
        if fields.len() > MAX_SCALAR_SLOTS {
            return Err("native scalar bridge state exceeds its slot bounds".to_owned());
        }
        let unique = fields
            .iter()
            .map(|(name, _)| name)
            .collect::<std::collections::BTreeSet<_>>();
        if unique.len() != fields.len() {
            return Err("native scalar bridge state contains duplicate field paths".to_owned());
        }
        Ok(Self {
            source: source.to_owned(),
            layout,
            fields,
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn fields(&self) -> &[(String, ScalarKind)] {
        &self.fields
    }

    /// Match the producer layout to the common registered session signature.
    /// No instruction spelling or backend-specific lowering policy lives here.
    pub fn bind(&self, module: &YirModule, id: &str) -> Result<[Vec<ScalarKind>; 3], String> {
        let registration = crate::registered_application_session(module, id)?;
        let signature = ApplicationSessionSignature::bind(module, registration.entries())?;
        if self.layout.type_name != signature.state_type
            || self.fields.len() != signature.state_parameters.len()
            || self
                .fields
                .iter()
                .zip(signature.state_parameters)
                .any(|((path, kind), p)| {
                    p.name != format!("{}{path}", signature.state_prefix)
                        || ScalarKind::parse(&p.ty).ok() != Some(*kind)
                })
        {
            return Err(
                "native scalar bridge state slots disagree with the flattened signature".to_owned(),
            );
        }
        let mut arguments = [Vec::new(), Vec::new(), Vec::new()];
        for (out, function) in
            arguments
                .iter_mut()
                .zip([signature.open, signature.event, signature.close])
        {
            if function.parameters.len() > MAX_SCALAR_SLOTS {
                return Err("native scalar bridge exceeds its argument bounds".to_owned());
            }
            *out = function
                .parameters
                .iter()
                .map(|p| ScalarKind::parse(&p.ty))
                .collect::<Result<_, _>>()?;
        }
        Ok(arguments)
    }

    pub fn unpack(&self, words: &[u64]) -> Result<Value, String> {
        if words.len() != self.fields.len() {
            return Err("native scalar bridge returned the wrong state slot count".to_owned());
        }
        let values = self
            .fields
            .iter()
            .zip(words)
            .map(|((_, kind), word)| kind.unpack(*word))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rebuild(&self.layout, &mut values.into_iter()))
    }
}

fn rebuild(layout: &OwnedStructLayout, values: &mut impl Iterator<Item = Value>) -> Value {
    Value::Struct(StructValue {
        type_name: layout.type_name.clone(),
        fields: layout
            .fields
            .iter()
            .map(|(name, field)| {
                let value = match field {
                    OwnedStructFieldLayout::Struct(nested) => rebuild(nested, values),
                    OwnedStructFieldLayout::Scalar(_) => {
                        values.next().expect("validated scalar layout")
                    }
                };
                (name.clone(), value)
            })
            .collect(),
    })
}

fn flatten(
    layout: &OwnedStructLayout,
    prefix: &str,
    out: &mut Vec<(String, ScalarKind)>,
) -> Result<(), String> {
    if layout.fields.is_empty() {
        return Err("native scalar bridge does not admit empty state structs".to_owned());
    }
    for (name, field) in &layout.fields {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        match field {
            OwnedStructFieldLayout::Struct(nested) => flatten(nested, &path, out)?,
            OwnedStructFieldLayout::Scalar(kind) => {
                let kind = match kind {
                    OwnedStructScalarLayout::Bool => ScalarKind::Bool,
                    OwnedStructScalarLayout::I32 => ScalarKind::I32,
                    OwnedStructScalarLayout::I64 => ScalarKind::I64,
                    OwnedStructScalarLayout::F32 => ScalarKind::F32,
                    OwnedStructScalarLayout::F64 => ScalarKind::F64,
                    _ => return Err("native scalar bridge state cannot carry resources".to_owned()),
                };
                out.push((path, kind));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_layout_reconstructs_nested_nominal_state_without_losing_float_bits() {
        let layout =
            ScalarStateLayout::parse("State{n:i32;child:Child{ready:bool;gain:f32;scale:f64}}")
                .unwrap();
        let words = [-17_i64 as u64, 1, 0x7fc0_1234, 0x7ff8_0000_0000_4321];
        let Value::Struct(state) = layout.unpack(&words).unwrap() else {
            panic!("state");
        };
        assert_eq!(state.type_name, "State");
        assert_eq!(state.fields[0], ("n".to_owned(), Value::I32(-17)));
        let Value::Struct(child) = &state.fields[1].1 else {
            panic!("child");
        };
        assert_eq!(child.type_name, "Child");
        assert_eq!(ScalarKind::F32.pack(&child.fields[1].1).unwrap(), words[2]);
        assert_eq!(ScalarKind::F64.pack(&child.fields[2].1).unwrap(), words[3]);
        for invalid in [vec![0; 3], vec![0, 2, 0, 0], vec![0, 1, u64::MAX, 0]] {
            assert!(layout.unpack(&invalid).is_err());
        }
        for invalid in [
            "Empty{}",
            "S{child:Empty{}}",
            "S{data:Bytes}",
            "S{x:i64;x:bool}",
        ] {
            assert!(ScalarStateLayout::parse(invalid).is_err());
        }
    }
}
