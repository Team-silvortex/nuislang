use crate::ArtifactError;

pub const COMPILER_STRUCTURAL_PROJECTION_CONTRACT: &str = "nuis-compiler-structural-projection-v1";
pub const COMPILER_AST_PROJECTION_ENCODING: &str = "nuis-ast-canonical-projection-v1";
pub const COMPILER_NIR_PROJECTION_ENCODING: &str = "nuis-nir-canonical-projection-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerProjectionKind {
    Ast,
    Nir,
}

impl CompilerProjectionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ast => "ast",
            Self::Nir => "nir",
        }
    }

    pub fn encoding(self) -> &'static str {
        match self {
            Self::Ast => COMPILER_AST_PROJECTION_ENCODING,
            Self::Nir => COMPILER_NIR_PROJECTION_ENCODING,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerProjectionRecordKind {
    ModuleDocumentation,
    Import,
    ModuleHeader,
    Item,
    Member,
    Nested,
    Documentation,
    OpaqueBody,
    OpaqueTerminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerProjectionRecord {
    pub ordinal: usize,
    pub depth: usize,
    pub kind: CompilerProjectionRecordKind,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerStructuralProjection {
    pub contract: String,
    pub kind: CompilerProjectionKind,
    pub module_domain: String,
    pub module_unit: String,
    pub records: Vec<CompilerProjectionRecord>,
}

pub fn parse_compiler_structural_projection(
    kind: CompilerProjectionKind,
    source: &str,
) -> Result<CompilerStructuralProjection, ArtifactError> {
    validate_text_contract(kind, source)?;

    let mut records = Vec::new();
    let mut module_identity = None;
    let mut imports_started = false;
    let mut previous_depth = 0usize;
    let mut opaque_leaf_open = false;

    for (ordinal, raw) in source.lines().enumerate() {
        if opaque_leaf_open {
            let record_kind = if raw.starts_with("  ") {
                CompilerProjectionRecordKind::OpaqueBody
            } else if raw.starts_with("})") {
                if raw.trim_end() != raw || raw.chars().any(char::is_control) {
                    return Err(projection_error(
                        kind,
                        ordinal,
                        "opaque leaf terminator is not canonical text",
                    ));
                }
                opaque_leaf_open = false;
                CompilerProjectionRecordKind::OpaqueTerminator
            } else {
                return Err(projection_error(
                    kind,
                    ordinal,
                    "opaque leaf body lines require two-space framing before a `})` terminator",
                ));
            };
            records.push(CompilerProjectionRecord {
                ordinal,
                depth: 0,
                kind: record_kind,
                body: raw.to_owned(),
            });
            continue;
        }

        let indentation = raw.bytes().take_while(|byte| *byte == b' ').count();
        if indentation % 2 != 0 {
            return Err(projection_error(
                kind,
                ordinal,
                "indentation must use complete two-space levels",
            ));
        }
        let depth = indentation / 2;
        let body = &raw[indentation..];
        validate_line(kind, ordinal, raw, body)?;

        let record_kind = if module_identity.is_none() {
            if depth != 0 {
                return Err(projection_error(
                    kind,
                    ordinal,
                    "module preamble records must have depth zero",
                ));
            }
            if body.starts_with("/// ") {
                if kind != CompilerProjectionKind::Ast || imports_started {
                    return Err(projection_error(
                        kind,
                        ordinal,
                        "module documentation is only valid before AST imports",
                    ));
                }
                CompilerProjectionRecordKind::ModuleDocumentation
            } else if body.starts_with("use ") {
                validate_import(kind, ordinal, body)?;
                imports_started = true;
                CompilerProjectionRecordKind::Import
            } else {
                module_identity = Some(parse_module_header(kind, ordinal, body)?);
                CompilerProjectionRecordKind::ModuleHeader
            }
        } else {
            if depth == 0 {
                return Err(projection_error(
                    kind,
                    ordinal,
                    "records after the module header must be structurally indented",
                ));
            }
            if depth > previous_depth + 1 {
                return Err(projection_error(
                    kind,
                    ordinal,
                    "record depth may increase by at most one level",
                ));
            }
            if body.starts_with("/// ") {
                if kind != CompilerProjectionKind::Ast {
                    return Err(projection_error(
                        kind,
                        ordinal,
                        "documentation records are not part of the NIR projection",
                    ));
                }
                CompilerProjectionRecordKind::Documentation
            } else {
                match depth {
                    1 => CompilerProjectionRecordKind::Item,
                    2 => CompilerProjectionRecordKind::Member,
                    _ => CompilerProjectionRecordKind::Nested,
                }
            }
        };

        previous_depth = depth;
        records.push(CompilerProjectionRecord {
            ordinal,
            depth,
            kind: record_kind,
            body: body.to_owned(),
        });
        opaque_leaf_open = kind == CompilerProjectionKind::Nir && opens_opaque_wgsl_leaf(body);
    }

    if opaque_leaf_open {
        return Err(ArtifactError::new(
            "compiler nir projection has an unterminated opaque WGSL leaf",
        ));
    }

    let (module_domain, module_unit) = module_identity.ok_or_else(|| {
        ArtifactError::new(format!(
            "compiler {} projection is missing its module header",
            kind.as_str()
        ))
    })?;
    let projection = CompilerStructuralProjection {
        contract: COMPILER_STRUCTURAL_PROJECTION_CONTRACT.to_owned(),
        kind,
        module_domain,
        module_unit,
        records,
    };
    if render_compiler_structural_projection(&projection) != source {
        return Err(ArtifactError::new(format!(
            "compiler {} projection is not canonically encoded",
            kind.as_str()
        )));
    }
    Ok(projection)
}

pub fn render_compiler_structural_projection(projection: &CompilerStructuralProjection) -> String {
    let mut out = String::new();
    for record in &projection.records {
        out.push_str(&"  ".repeat(record.depth));
        out.push_str(&record.body);
        out.push('\n');
    }
    out
}

pub fn verify_compiler_projection_identity(
    projection: &CompilerStructuralProjection,
    expected_domain: &str,
    expected_unit: &str,
) -> Result<(), ArtifactError> {
    if projection.module_domain != expected_domain || projection.module_unit != expected_unit {
        return Err(ArtifactError::new(format!(
            "compiler {} projection module `{}::{}` does not match handoff module `{}::{}`",
            projection.kind.as_str(),
            projection.module_domain,
            projection.module_unit,
            expected_domain,
            expected_unit
        )));
    }
    Ok(())
}

fn validate_text_contract(kind: CompilerProjectionKind, source: &str) -> Result<(), ArtifactError> {
    if source.is_empty() || !source.ends_with('\n') {
        return Err(ArtifactError::new(format!(
            "compiler {} projection must be non-empty and end with LF",
            kind.as_str()
        )));
    }
    if source.contains(['\r', '\0']) {
        return Err(ArtifactError::new(format!(
            "compiler {} projection violates the UTF-8/LF text contract",
            kind.as_str()
        )));
    }
    Ok(())
}

fn validate_line(
    kind: CompilerProjectionKind,
    ordinal: usize,
    raw: &str,
    body: &str,
) -> Result<(), ArtifactError> {
    if body.is_empty() {
        return Err(projection_error(
            kind,
            ordinal,
            "empty records are forbidden",
        ));
    }
    if raw.trim_end() != raw {
        return Err(projection_error(
            kind,
            ordinal,
            "trailing whitespace is not canonical",
        ));
    }
    if body.chars().any(char::is_control) {
        return Err(projection_error(
            kind,
            ordinal,
            "record bodies cannot contain control characters",
        ));
    }
    Ok(())
}

fn validate_import(
    kind: CompilerProjectionKind,
    ordinal: usize,
    body: &str,
) -> Result<(), ArtifactError> {
    let fields = body.split(' ').collect::<Vec<_>>();
    if fields.len() != 3 || fields[0] != "use" {
        return Err(projection_error(
            kind,
            ordinal,
            "imports must use `use <domain> <unit>`",
        ));
    }
    validate_atom(kind, ordinal, "import domain", fields[1])?;
    validate_atom(kind, ordinal, "import unit", fields[2])
}

fn parse_module_header(
    kind: CompilerProjectionKind,
    ordinal: usize,
    body: &str,
) -> Result<(String, String), ArtifactError> {
    let fields = body.split(' ').collect::<Vec<_>>();
    if fields.len() != 5 || fields[0] != kind.as_str() || fields[1] != "mod" || fields[3] != "unit"
    {
        return Err(projection_error(
            kind,
            ordinal,
            &format!(
                "module header must use `{} mod <domain> unit <unit>`",
                kind.as_str()
            ),
        ));
    }
    validate_atom(kind, ordinal, "module domain", fields[2])?;
    validate_atom(kind, ordinal, "module unit", fields[4])?;
    Ok((fields[2].to_owned(), fields[4].to_owned()))
}

fn validate_atom(
    kind: CompilerProjectionKind,
    ordinal: usize,
    label: &str,
    value: &str,
) -> Result<(), ArtifactError> {
    if value.is_empty() || value.chars().any(|character| character.is_whitespace()) {
        return Err(projection_error(
            kind,
            ordinal,
            &format!("{label} must be one non-empty atom"),
        ));
    }
    Ok(())
}

fn opens_opaque_wgsl_leaf(body: &str) -> bool {
    body.contains("shader_inline_wgsl(") && body.ends_with(", wgsl {")
}

fn projection_error(kind: CompilerProjectionKind, ordinal: usize, message: &str) -> ArtifactError {
    ArtifactError::new(format!(
        "compiler {} projection record {ordinal}: {message}",
        kind.as_str()
    ))
}

#[cfg(test)]
#[path = "compiler_structural_projection_tests.rs"]
mod tests;
