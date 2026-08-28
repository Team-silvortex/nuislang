use crate::ArtifactError;

pub const COMPILER_TOKEN_DECODER_CONTRACT: &str = "nuis-compiler-token-decoder-v1";
pub const COMPILER_TOKEN_STREAM_PROTOCOL: &str = "nuis-token-stream-v1";
pub const COMPILER_TOKEN_DECODER_MAX_BYTES: usize = 4 * 1024 * 1024;
pub const COMPILER_TOKEN_DECODER_MAX_RECORDS: usize = 65_535;
pub const COMPILER_TOKEN_DECODER_SEMANTIC_SEED: usize = 313;
pub const COMPILER_TOKEN_DECODER_FOLD_MODULUS: usize = 2_147_483_629;
pub const COMPILER_TOKEN_PAGE_RECORDS: usize = 4;
pub const COMPILER_TOKEN_PAGE_PAYLOAD_BYTES: usize = 64;
pub const COMPILER_TOKEN_PAGE_CANONICAL_BYTES: usize = 128;
pub const COMPILER_TOKEN_PAGE_IDENTITY_RADIX: usize = 129;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilerTokenDecodeSummary {
    pub record_count: usize,
    pub semantic_fold: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilerTokenPageIdentity {
    pub record_count: usize,
    pub payload_bytes: usize,
    pub canonical_bytes: usize,
    pub canonical_hash: usize,
    pub identity: usize,
}

#[derive(Debug, Clone, Copy)]
enum TokenKind {
    Word,
    Integer,
    Float,
    Symbol,
    Arrow,
    String,
    DocComment,
}

impl TokenKind {
    fn code(self) -> usize {
        match self {
            Self::Word => 1,
            Self::Integer => 2,
            Self::Float => 3,
            Self::Symbol => 4,
            Self::Arrow => 5,
            Self::String => 6,
            Self::DocComment => 7,
        }
    }
}

pub fn decode_compiler_token_stream(
    bytes: &[u8],
) -> Result<CompilerTokenDecodeSummary, ArtifactError> {
    if bytes.is_empty() || bytes.len() > COMPILER_TOKEN_DECODER_MAX_BYTES {
        return Err(ArtifactError::new(format!(
            "compiler token stream must contain 1..={COMPILER_TOKEN_DECODER_MAX_BYTES} bytes"
        )));
    }
    let source = std::str::from_utf8(bytes).map_err(|error| {
        ArtifactError::new(format!("compiler token stream is not UTF-8: {error}"))
    })?;
    if !source.ends_with('\n') || source.contains('\r') || source.contains('\0') {
        return Err(ArtifactError::new(
            "compiler token stream must use canonical UTF-8/LF text",
        ));
    }
    let mut lines = source.split_terminator('\n');
    if lines.next() != Some(COMPILER_TOKEN_STREAM_PROTOCOL) {
        return Err(ArtifactError::new(format!(
            "compiler token stream must begin with `{COMPILER_TOKEN_STREAM_PROTOCOL}`"
        )));
    }

    let mut summary = CompilerTokenDecodeSummary {
        record_count: 0,
        semantic_fold: COMPILER_TOKEN_DECODER_SEMANTIC_SEED,
    };
    for line in lines {
        if summary.record_count == COMPILER_TOKEN_DECODER_MAX_RECORDS {
            return Err(ArtifactError::new(format!(
                "compiler token stream exceeds {COMPILER_TOKEN_DECODER_MAX_RECORDS} records"
            )));
        }
        let (kind, payload) = parse_record(line)?;
        summary.semantic_fold = fold_payload(summary.semantic_fold, kind, payload)?;
        summary.semantic_fold = fold_unit(summary.semantic_fold, 128 + kind.code());
        summary.record_count += 1;
    }
    Ok(summary)
}

pub fn compiler_token_first_page_identity(
    bytes: &[u8],
) -> Result<CompilerTokenPageIdentity, ArtifactError> {
    let summary = decode_compiler_token_stream(bytes)?;
    if summary.record_count < COMPILER_TOKEN_PAGE_RECORDS {
        return Err(ArtifactError::new(format!(
            "compiler token page requires at least {COMPILER_TOKEN_PAGE_RECORDS} records"
        )));
    }
    let source = std::str::from_utf8(bytes).map_err(|error| {
        ArtifactError::new(format!("compiler token stream is not UTF-8: {error}"))
    })?;
    let mut lines = source.split_terminator('\n');
    let header = lines
        .next()
        .ok_or_else(|| ArtifactError::new("compiler token page is missing its protocol header"))?;
    let mut canonical = Vec::from(header.as_bytes());
    canonical.push(b'\n');
    let mut payload_bytes = 0;
    for _ in 0..COMPILER_TOKEN_PAGE_RECORDS {
        let line = lines
            .next()
            .ok_or_else(|| ArtifactError::new("compiler token page ended before four records"))?;
        canonicalize_page_record(line, &mut canonical, &mut payload_bytes)?;
    }
    let raw_page_bytes = bytes
        .iter()
        .enumerate()
        .filter(|(_, byte)| **byte == b'\n')
        .nth(COMPILER_TOKEN_PAGE_RECORDS)
        .map(|(index, _)| index + 1)
        .ok_or_else(|| ArtifactError::new("compiler token page has no fourth record boundary"))?;
    if raw_page_bytes > COMPILER_TOKEN_PAGE_CANONICAL_BYTES
        || payload_bytes > COMPILER_TOKEN_PAGE_PAYLOAD_BYTES
        || canonical.len() > COMPILER_TOKEN_PAGE_CANONICAL_BYTES
    {
        return Err(ArtifactError::new(
            "compiler token page exceeds the bounded materializer capacity",
        ));
    }
    let canonical_hash = canonical
        .iter()
        .fold(COMPILER_TOKEN_DECODER_SEMANTIC_SEED, |state, byte| {
            fold_unit(state, usize::from(*byte) + 1)
        });
    let identity = canonical_hash * COMPILER_TOKEN_PAGE_IDENTITY_RADIX + canonical.len();
    Ok(CompilerTokenPageIdentity {
        record_count: COMPILER_TOKEN_PAGE_RECORDS,
        payload_bytes,
        canonical_bytes: canonical.len(),
        canonical_hash,
        identity,
    })
}

fn canonicalize_page_record(
    line: &str,
    output: &mut Vec<u8>,
    payload_bytes: &mut usize,
) -> Result<(), ArtifactError> {
    let (kind, payload) = parse_record(line)?;
    match kind {
        TokenKind::Word | TokenKind::Float | TokenKind::String | TokenKind::DocComment => {
            fold_hex_payload(
                0,
                payload,
                matches!(kind, TokenKind::String | TokenKind::DocComment),
            )?;
            *payload_bytes += payload.len() / 2;
            output.extend_from_slice(match kind {
                TokenKind::Word => b"word\t",
                TokenKind::Float => b"float\t",
                TokenKind::String => b"string\t",
                TokenKind::DocComment => b"doc-comment\t",
                _ => unreachable!(),
            });
            output.extend_from_slice(payload.as_bytes());
        }
        TokenKind::Integer => {
            let value = payload.parse::<i64>().map_err(|error| {
                ArtifactError::new(format!(
                    "invalid compiler integer token `{payload}`: {error}"
                ))
            })?;
            output.extend_from_slice(b"integer\t");
            output.extend_from_slice(value.to_string().as_bytes());
        }
        TokenKind::Symbol => {
            let scalar = payload.parse::<usize>().map_err(|error| {
                ArtifactError::new(format!(
                    "invalid compiler symbol token `{payload}`: {error}"
                ))
            })?;
            if scalar > 0x10ffff || (0xd800..=0xdfff).contains(&scalar) {
                return Err(ArtifactError::new(format!(
                    "compiler symbol token `{payload}` is not a Unicode scalar"
                )));
            }
            output.extend_from_slice(b"symbol\t");
            output.extend_from_slice(scalar.to_string().as_bytes());
        }
        TokenKind::Arrow => output.extend_from_slice(b"arrow"),
    }
    output.push(b'\n');
    Ok(())
}

fn parse_record(line: &str) -> Result<(TokenKind, &str), ArtifactError> {
    if line == "arrow" {
        return Ok((TokenKind::Arrow, ""));
    }
    let (kind, payload) = line
        .split_once('\t')
        .ok_or_else(|| ArtifactError::new(format!("malformed compiler token record `{line}`")))?;
    let kind = match kind {
        "word" => TokenKind::Word,
        "integer" => TokenKind::Integer,
        "float" => TokenKind::Float,
        "symbol" => TokenKind::Symbol,
        "string" => TokenKind::String,
        "doc-comment" => TokenKind::DocComment,
        _ => {
            return Err(ArtifactError::new(format!(
                "unsupported compiler token record kind `{kind}`"
            )))
        }
    };
    Ok((kind, payload))
}

fn fold_payload(state: usize, kind: TokenKind, payload: &str) -> Result<usize, ArtifactError> {
    match kind {
        TokenKind::Word | TokenKind::Float => fold_hex_payload(state, payload, false),
        TokenKind::String | TokenKind::DocComment => fold_hex_payload(state, payload, true),
        TokenKind::Integer => fold_integer_payload(state, payload),
        TokenKind::Symbol => fold_symbol_payload(state, payload),
        TokenKind::Arrow => Ok(state),
    }
}

fn fold_hex_payload(
    mut state: usize,
    payload: &str,
    allow_empty: bool,
) -> Result<usize, ArtifactError> {
    if (!allow_empty && payload.is_empty()) || !payload.len().is_multiple_of(2) {
        return Err(ArtifactError::new(
            "compiler token hexadecimal payload has invalid length",
        ));
    }
    for pair in payload.as_bytes().chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        state = fold_unit(state, usize::from(high) + 1);
        state = fold_unit(state, usize::from(low) + 1);
    }
    Ok(state)
}

fn fold_integer_payload(mut state: usize, payload: &str) -> Result<usize, ArtifactError> {
    let digits = payload.strip_prefix('-').unwrap_or(payload);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ArtifactError::new(format!(
            "invalid compiler integer token `{payload}`"
        )));
    }
    for byte in payload.bytes() {
        let unit = if byte == b'-' {
            17
        } else {
            usize::from(byte - b'0') + 1
        };
        state = fold_unit(state, unit);
    }
    Ok(state)
}

fn fold_symbol_payload(mut state: usize, payload: &str) -> Result<usize, ArtifactError> {
    if payload.is_empty() || !payload.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ArtifactError::new(format!(
            "invalid compiler symbol token `{payload}`"
        )));
    }
    for byte in payload.bytes() {
        state = fold_unit(state, usize::from(byte - b'0') + 1);
    }
    Ok(state)
}

fn fold_unit(state: usize, unit: usize) -> usize {
    (((state as u64 * 257) + unit as u64) % COMPILER_TOKEN_DECODER_FOLD_MODULUS as u64) as usize
}

fn hex_nibble(byte: u8) -> Result<u8, ArtifactError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(ArtifactError::new(format!(
            "compiler token hexadecimal payload contains invalid byte `{}`",
            char::from(byte)
        ))),
    }
}

#[cfg(test)]
#[path = "compiler_token_decoder_tests.rs"]
mod tests;
