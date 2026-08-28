use crate::{parse_compiler_structural_projection, ArtifactError, CompilerProjectionKind};

pub const COMPILER_PROJECTION_PAGE_CONTRACT: &str = "nuis-compiler-structural-page-v1";
pub const COMPILER_PROJECTION_PAGE_BYTES: usize = 128;
pub const COMPILER_PROJECTION_PAGE_BODY_HASH_SEED: usize = 431;
pub const COMPILER_PROJECTION_PAGE_HASH_SEED: usize = 421;
pub const COMPILER_PROJECTION_PAGE_HASH_MODULUS: usize = 2_147_483_647;
pub const COMPILER_PROJECTION_PAGE_IDENTITY_RADIX: usize = 129;

const PROJECTION_HASH_MODULUS: u64 = COMPILER_PROJECTION_PAGE_HASH_MODULUS as u64;
const MODULE_DOCUMENTATION_PREFIX: usize = 0x202f_2f2f;
const IMPORT_PREFIX: usize = 0x2065_7375;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilerProjectionPageIdentity {
    pub record_count: usize,
    pub page_bytes: usize,
    pub projection_hash: usize,
    pub continuation_indentation: usize,
    pub continuation_body_bytes: usize,
    pub continuation_body_hash: usize,
    pub state_hash: usize,
    pub identity: usize,
}

#[derive(Debug, Clone, Copy)]
struct ProjectionState {
    is_ast: usize,
    module_seen: usize,
    imports_started: usize,
    previous_depth: usize,
    opaque_leaf_open: usize,
    record_count: usize,
    hash: usize,
}

#[derive(Debug, Clone, Copy)]
struct PageState {
    projection: ProjectionState,
    indentation: usize,
    body_length: usize,
    body_hash: usize,
    prefix: usize,
    last_byte: usize,
    line_open: bool,
    input_bytes: usize,
}

pub fn compiler_projection_first_page_identity(
    kind: CompilerProjectionKind,
    bytes: &[u8],
) -> Result<CompilerProjectionPageIdentity, ArtifactError> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        ArtifactError::new(format!(
            "compiler {} projection is not UTF-8: {error}",
            kind.as_str()
        ))
    })?;
    parse_compiler_structural_projection(kind, source)?;

    let page_bytes = bytes.len().min(COMPILER_PROJECTION_PAGE_BYTES);
    let mut state = PageState::new(kind);
    for byte in &bytes[..page_bytes] {
        state = state.step(*byte)?;
    }
    if state.projection.record_count == 0 {
        return Err(ArtifactError::new(
            "compiler structural page requires at least one complete record",
        ));
    }
    let state_hash = state.state_hash();
    Ok(CompilerProjectionPageIdentity {
        record_count: state.projection.record_count,
        page_bytes,
        projection_hash: state.projection.hash,
        continuation_indentation: state.indentation,
        continuation_body_bytes: state.body_length,
        continuation_body_hash: state.body_hash,
        state_hash,
        identity: state_hash * COMPILER_PROJECTION_PAGE_IDENTITY_RADIX + page_bytes,
    })
}

impl PageState {
    fn new(kind: CompilerProjectionKind) -> Self {
        Self {
            projection: ProjectionState {
                is_ast: usize::from(kind == CompilerProjectionKind::Ast),
                module_seen: 0,
                imports_started: 0,
                previous_depth: 0,
                opaque_leaf_open: 0,
                record_count: 0,
                hash: 0,
            },
            indentation: 0,
            body_length: 0,
            body_hash: COMPILER_PROJECTION_PAGE_BODY_HASH_SEED,
            prefix: 0,
            last_byte: 0,
            line_open: false,
            input_bytes: 0,
        }
    }

    fn step(mut self, byte: u8) -> Result<Self, ArtifactError> {
        if byte == 0 || byte == b'\r' {
            return Err(ArtifactError::new(
                "compiler structural page contains a forbidden byte",
            ));
        }
        self.input_bytes += 1;
        if byte == b'\n' {
            return self.finish_line();
        }
        if !self.line_open && byte == b' ' {
            self.indentation += 1;
            return Ok(self);
        }
        if !self.line_open {
            if !self.indentation.is_multiple_of(2) {
                return Err(ArtifactError::new(
                    "compiler structural page has incomplete indentation",
                ));
            }
            self.line_open = true;
        }
        if byte.is_ascii_control() {
            return Err(ArtifactError::new(
                "compiler structural page body contains a control byte",
            ));
        }
        if self.body_length < 4 {
            self.prefix += usize::from(byte) << (self.body_length * 8);
        }
        self.body_hash = page_fold(self.body_hash, usize::from(byte) + 1);
        self.body_length += 1;
        self.last_byte = usize::from(byte);
        Ok(self)
    }

    fn finish_line(mut self) -> Result<Self, ArtifactError> {
        if !self.line_open
            || !self.indentation.is_multiple_of(2)
            || matches!(self.last_byte, 9 | 32)
        {
            return Err(ArtifactError::new(
                "compiler structural page line is not canonical",
            ));
        }
        let depth = self.indentation / 2;
        let code = self.record_code(depth)?;
        self.projection = self.projection.accept(code, depth, self.body_hash, false)?;
        self.indentation = 0;
        self.body_length = 0;
        self.body_hash = COMPILER_PROJECTION_PAGE_BODY_HASH_SEED;
        self.prefix = 0;
        self.last_byte = 0;
        self.line_open = false;
        Ok(self)
    }

    fn record_code(self, depth: usize) -> Result<usize, ArtifactError> {
        if self.projection.module_seen == 0 {
            if self.prefix == MODULE_DOCUMENTATION_PREFIX {
                return Ok(1);
            }
            if self.prefix == IMPORT_PREFIX {
                return Ok(2);
            }
            return Ok(3);
        }
        if self.prefix == MODULE_DOCUMENTATION_PREFIX {
            return Ok(7);
        }
        match depth {
            1 => Ok(4),
            2 => Ok(5),
            3.. => Ok(6),
            _ => Err(ArtifactError::new(
                "compiler structural page body record has invalid depth",
            )),
        }
    }

    fn state_hash(self) -> usize {
        [
            self.projection.is_ast,
            self.projection.module_seen,
            self.projection.imports_started,
            self.projection.previous_depth,
            self.projection.opaque_leaf_open,
            self.projection.record_count,
            self.projection.hash,
            self.indentation,
            self.body_length,
            self.body_hash,
            self.prefix,
            self.last_byte,
            usize::from(self.line_open),
            self.input_bytes,
        ]
        .into_iter()
        .fold(COMPILER_PROJECTION_PAGE_HASH_SEED, page_fold)
    }
}

impl ProjectionState {
    fn accept(
        mut self,
        code: usize,
        depth: usize,
        body_hash: usize,
        opens_opaque: bool,
    ) -> Result<Self, ArtifactError> {
        if !(1..=9).contains(&code) || opens_opaque || self.opaque_leaf_open != 0 {
            return Err(page_structure_error());
        }
        let next_hash = projection_hash(self.hash, code, depth, body_hash);
        if self.module_seen == 0 {
            if depth != 0 {
                return Err(page_structure_error());
            }
            match code {
                1 if self.is_ast != 0 && self.imports_started == 0 => {}
                2 => self.imports_started = 1,
                3 => self.module_seen = 1,
                _ => return Err(page_structure_error()),
            }
        } else {
            if code <= 3
                || code >= 8
                || depth == 0
                || depth > self.previous_depth + 1
                || (code == 4 && depth != 1)
                || (code == 5 && depth != 2)
                || (code == 6 && depth < 3)
                || (code == 7 && self.is_ast == 0)
            {
                return Err(page_structure_error());
            }
            self.previous_depth = depth;
        }
        self.record_count += 1;
        self.hash = next_hash;
        Ok(self)
    }
}

fn projection_hash(hash: usize, code: usize, depth: usize, body_hash: usize) -> usize {
    ((hash as u64 * 65_599 + code as u64 * 257 + depth as u64 * 17 + body_hash as u64 + 1)
        % PROJECTION_HASH_MODULUS) as usize
}

fn page_fold(state: usize, value: usize) -> usize {
    ((state as u64 * 257 + value as u64 + 1) % PROJECTION_HASH_MODULUS) as usize
}

fn page_structure_error() -> ArtifactError {
    ArtifactError::new("compiler structural page violates projection ordering")
}

#[cfg(test)]
#[path = "compiler_structural_projection_page_tests.rs"]
mod tests;
