pub const OWNED_VARIANT_UNION_LAYOUT_PREFIX: &str = "__nuis_variant_union__";

const MAX_LAYOUT_BYTES: usize = 64 * 1024;
const MAX_LAYOUT_DEPTH: usize = 64;
const MAX_LAYOUT_FIELDS: usize = 4096;
const MAX_LAYOUT_NAME_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnedStructScalarLayout {
    Bool,
    I32,
    I64,
    F32,
    F64,
    String,
    Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedStructFieldLayout {
    Scalar(OwnedStructScalarLayout),
    Struct(OwnedStructLayout),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedStructLayout {
    pub type_name: String,
    pub fields: Vec<(String, OwnedStructFieldLayout)>,
}

pub fn parse_owned_struct_layout(source: &str) -> Result<OwnedStructLayout, String> {
    if source.is_empty() || source.len() > MAX_LAYOUT_BYTES {
        return Err(format!(
            "owned struct layout must contain 1..={MAX_LAYOUT_BYTES} bytes"
        ));
    }
    let mut parser = OwnedStructLayoutParser {
        source: source.as_bytes(),
        position: 0,
        field_count: 0,
    };
    let parsed = parser.parse_struct(0)?;
    if parser.position != parser.source.len() {
        return Err(format!("trailing data in owned struct layout `{source}`"));
    }
    Ok(parsed)
}

pub fn owned_struct_scalar_leaf_count(layout: &OwnedStructLayout) -> usize {
    if layout.fields.is_empty() {
        return 1;
    }
    layout
        .fields
        .iter()
        .map(|(_, field)| match field {
            OwnedStructFieldLayout::Scalar(_) => 1,
            OwnedStructFieldLayout::Struct(nested) => owned_struct_scalar_leaf_count(nested),
        })
        .sum()
}

struct OwnedStructLayoutParser<'a> {
    source: &'a [u8],
    position: usize,
    field_count: usize,
}

impl OwnedStructLayoutParser<'_> {
    fn parse_struct(&mut self, depth: usize) -> Result<OwnedStructLayout, String> {
        let type_name = self.parse_name()?;
        self.parse_struct_body(type_name, depth)
    }

    fn parse_struct_body(
        &mut self,
        type_name: String,
        depth: usize,
    ) -> Result<OwnedStructLayout, String> {
        if depth >= MAX_LAYOUT_DEPTH {
            return Err(format!(
                "owned struct layout exceeds maximum depth {MAX_LAYOUT_DEPTH}"
            ));
        }
        self.expect(b'{')?;
        let mut fields = Vec::new();
        loop {
            if self.consume(b'}') {
                break;
            }
            self.field_count += 1;
            if self.field_count > MAX_LAYOUT_FIELDS {
                return Err(format!(
                    "owned struct layout exceeds maximum field count {MAX_LAYOUT_FIELDS}"
                ));
            }
            let name = self.parse_name()?;
            self.expect(b':')?;
            let kind = self.parse_name()?;
            let value = match parse_scalar_layout(&kind) {
                Some(kind) => OwnedStructFieldLayout::Scalar(kind),
                None => OwnedStructFieldLayout::Struct(self.parse_struct_body(kind, depth + 1)?),
            };
            fields.push((name, value));
            if self.consume(b'}') {
                break;
            }
            self.expect(b';')?;
        }
        Ok(OwnedStructLayout { type_name, fields })
    }

    fn parse_name(&mut self) -> Result<String, String> {
        let start = self.position;
        while self.position < self.source.len()
            && !matches!(self.source[self.position], b'{' | b'}' | b':' | b';')
        {
            self.position += 1;
        }
        let len = self.position.saturating_sub(start);
        if len == 0 || len > MAX_LAYOUT_NAME_BYTES {
            return Err(format!(
                "owned struct layout names must contain 1..={MAX_LAYOUT_NAME_BYTES} bytes"
            ));
        }
        String::from_utf8(self.source[start..self.position].to_vec())
            .map_err(|_| "owned struct layout names must be UTF-8".to_owned())
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.consume(byte) {
            Ok(())
        } else {
            Err(format!(
                "expected `{}` in owned struct layout",
                byte as char
            ))
        }
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.source.get(self.position) == Some(&byte) {
            self.position += 1;
            true
        } else {
            false
        }
    }
}

fn parse_scalar_layout(kind: &str) -> Option<OwnedStructScalarLayout> {
    match kind {
        "bool" => Some(OwnedStructScalarLayout::Bool),
        "i32" => Some(OwnedStructScalarLayout::I32),
        "i64" => Some(OwnedStructScalarLayout::I64),
        "f32" => Some(OwnedStructScalarLayout::F32),
        "f64" => Some(OwnedStructScalarLayout::F64),
        "String" => Some(OwnedStructScalarLayout::String),
        "Bytes" => Some(OwnedStructScalarLayout::Bytes),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_nested_and_variant_union_layouts() {
        let flat = parse_owned_struct_layout("Summary{ready:bool;score:i64}").unwrap();
        assert_eq!(flat.type_name, "Summary");
        assert_eq!(flat.fields.len(), 2);

        let nested =
            parse_owned_struct_layout("Outer{inner:Inner{label:String;payload:Bytes};value:f64}")
                .unwrap();
        assert!(matches!(
            nested.fields[0].1,
            OwnedStructFieldLayout::Struct(_)
        ));

        let union = parse_owned_struct_layout(
            "__nuis_variant_union__Result{tag:i64;Result.Ok:Result.Ok{value:i64};Result.Err:Result.Err{}}",
        )
        .unwrap();
        assert_eq!(union.fields.len(), 3);
    }

    #[test]
    fn rejects_trailing_or_malformed_layouts() {
        assert!(parse_owned_struct_layout("Summary{score:i64}tail").is_err());
        assert!(parse_owned_struct_layout("Summary{score:i64").is_err());
        assert!(parse_owned_struct_layout("").is_err());
    }

    #[test]
    fn counts_nested_scalar_layout_leaves() {
        let layout =
            parse_owned_struct_layout("Outer{ready:bool;inner:Inner{x:i64;y:f32}}").unwrap();
        assert_eq!(owned_struct_scalar_leaf_count(&layout), 3);
    }
}
