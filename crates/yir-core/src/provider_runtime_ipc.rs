use std::io::{Read, Write};

#[path = "provider_runtime_arguments.rs"]
mod arguments;
pub use arguments::{DispatchArguments, DispatchResource};

pub const CONTRACT: &str = "nuis-yir-provider-runtime-ipc-v2";
pub const SOCKET_ENV: &str = "NUIS_YIR_PROVIDER_DISPATCH_SOCKET";
pub const MAX_DISPATCHES: usize = 256;
pub const MAX_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
const MAX_HEADER_BYTES: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchTarget {
    pub source_yir_fnv1a64: String,
    pub module: String,
    pub instruction: String,
    pub node: String,
    pub resource: String,
}

impl DispatchTarget {
    pub fn matches(&self, node: &crate::Node) -> bool {
        self.module == node.op.module
            && self.instruction == node.op.instruction
            && self.node == node.name
            && self.resource == node.resource
    }

    fn fields(&self) -> [&str; 5] {
        [
            &self.source_yir_fnv1a64,
            &self.module,
            &self.instruction,
            &self.node,
            &self.resource,
        ]
    }

    fn parse(fields: &[&str]) -> Result<Self, String> {
        if fields.len() != 5 || !valid_hash(fields[0]) {
            return Err("runtime IPC target identity is invalid".to_owned());
        }
        Ok(Self {
            source_yir_fnv1a64: fields[0].to_owned(),
            module: fields[1].to_owned(),
            instruction: fields[2].to_owned(),
            node: fields[3].to_owned(),
            resource: fields[4].to_owned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchFrame {
    pub sequence: usize,
    pub arguments: DispatchArguments,
    pub request_id: String,
    pub provider_family: String,
    pub element_type: String,
    pub layout: String,
    pub shape: Vec<usize>,
    pub row_stride_bytes: usize,
    pub payload: Vec<u8>,
    pub completion_wire: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Hello(DispatchTarget),
    Dispatch {
        sequence: usize,
        target: DispatchTarget,
        arguments: DispatchArguments,
    },
    Frame(DispatchFrame),
    Finish(usize),
    Closed(usize),
    Rejected(String),
}

impl Message {
    pub fn write_to(&self, writer: &mut impl Write) -> Result<(), String> {
        let mut fields = vec![CONTRACT.to_owned()];
        let mut payload: &[u8] = &[];
        match self {
            Self::Hello(target) => {
                DispatchTarget::parse(&target.fields())?;
                fields.push("hello".to_owned());
                fields.extend(target.fields().map(str::to_owned));
            }
            Self::Dispatch {
                sequence,
                target,
                arguments,
            } => {
                DispatchTarget::parse(&target.fields())?;
                fields.extend(["dispatch".to_owned(), sequence.to_string()]);
                fields.extend(target.fields().map(str::to_owned));
                fields.push(arguments.to_wire()?);
            }
            Self::Frame(frame) => {
                validate_frame(frame)?;
                fields.extend([
                    "frame".to_owned(),
                    frame.sequence.to_string(),
                    frame.request_id.clone(),
                    frame.provider_family.clone(),
                    frame.element_type.clone(),
                    frame.layout.clone(),
                    frame
                        .shape
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join("x"),
                    frame.row_stride_bytes.to_string(),
                    frame.payload.len().to_string(),
                    hash_bytes(&frame.payload),
                    frame.completion_wire.clone(),
                    frame.arguments.to_wire()?,
                ]);
                payload = &frame.payload;
            }
            Self::Finish(sequence) => fields.extend(["finish".to_owned(), sequence.to_string()]),
            Self::Closed(sequence) => fields.extend(["closed".to_owned(), sequence.to_string()]),
            Self::Rejected(error) => fields.extend(["rejected".to_owned(), error.clone()]),
        }
        if fields.iter().any(|value| !valid_field(value)) {
            return Err("runtime IPC field is invalid".to_owned());
        }
        let header = fields.join("\n");
        if header.len() > MAX_HEADER_BYTES {
            return Err("runtime IPC header exceeds limit".to_owned());
        }
        writer
            .write_all(&(header.len() as u32).to_le_bytes())
            .and_then(|_| writer.write_all(header.as_bytes()))
            .and_then(|_| writer.write_all(payload))
            .and_then(|_| writer.flush())
            .map_err(|error| format!("runtime IPC write failed: {error}"))
    }

    pub fn read_from(reader: &mut impl Read) -> Result<Self, String> {
        let mut size = [0u8; 4];
        read_exact(reader, &mut size)?;
        let size = u32::from_le_bytes(size) as usize;
        if size == 0 || size > MAX_HEADER_BYTES {
            return Err("runtime IPC header size is invalid".to_owned());
        }
        let mut header = vec![0; size];
        read_exact(reader, &mut header)?;
        let header = std::str::from_utf8(&header)
            .map_err(|_| "runtime IPC header is not UTF-8".to_owned())?;
        let fields = header.split('\n').collect::<Vec<_>>();
        if fields.iter().any(|field| !valid_field(field)) || fields[0] != CONTRACT {
            return Err("runtime IPC contract or fields are invalid".to_owned());
        }
        match fields.get(1).copied() {
            Some("hello") if fields.len() == 7 => {
                Ok(Self::Hello(DispatchTarget::parse(&fields[2..])?))
            }
            Some("dispatch") if fields.len() == 9 => Ok(Self::Dispatch {
                sequence: number(fields[2])?,
                target: DispatchTarget::parse(&fields[3..8])?,
                arguments: DispatchArguments::parse(fields[8])?,
            }),
            Some("finish") if fields.len() == 3 => Ok(Self::Finish(number(fields[2])?)),
            Some("closed") if fields.len() == 3 => Ok(Self::Closed(number(fields[2])?)),
            Some("rejected") if fields.len() == 3 => Ok(Self::Rejected(fields[2].to_owned())),
            Some("frame") if fields.len() == 13 => {
                let length = number(fields[9])?;
                if length == 0 || length > MAX_PAYLOAD_BYTES {
                    return Err("runtime IPC payload size is invalid".to_owned());
                }
                let mut payload = vec![0; length];
                read_exact(reader, &mut payload)?;
                if hash_bytes(&payload) != fields[10] {
                    return Err("runtime IPC payload identity mismatch".to_owned());
                }
                let frame = DispatchFrame {
                    sequence: number(fields[2])?,
                    arguments: DispatchArguments::parse(fields[12])?,
                    request_id: fields[3].to_owned(),
                    provider_family: fields[4].to_owned(),
                    element_type: fields[5].to_owned(),
                    layout: fields[6].to_owned(),
                    shape: fields[7].split('x').map(number).collect::<Result<_, _>>()?,
                    row_stride_bytes: number(fields[8])?,
                    payload,
                    completion_wire: fields[11].to_owned(),
                };
                validate_frame(&frame)?;
                Ok(Self::Frame(frame))
            }
            _ => Err("runtime IPC message shape is invalid".to_owned()),
        }
    }
}

fn validate_frame(frame: &DispatchFrame) -> Result<(), String> {
    if frame.sequence >= MAX_DISPATCHES
        || frame.shape.is_empty()
        || frame.shape.len() > 8
        || frame.shape.contains(&0)
        || frame.row_stride_bytes == 0
        || frame.payload.is_empty()
        || frame.payload.len() > MAX_PAYLOAD_BYTES
    {
        return Err("runtime IPC result descriptor is invalid".to_owned());
    }
    crate::ProviderPhysicalCompletion::parse(&frame.completion_wire)?;
    Ok(())
}

fn valid_field(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && !value.contains(['\n', '\r', '\0'])
}

fn number(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|parsed| parsed.to_string() == value)
        .ok_or_else(|| "runtime IPC integer is invalid".to_owned())
}

fn read_exact(reader: &mut impl Read, bytes: &mut [u8]) -> Result<(), String> {
    reader
        .read_exact(bytes)
        .map_err(|error| format!("runtime IPC read failed: {error}"))
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

fn valid_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[cfg(test)]
#[path = "provider_runtime_ipc_tests.rs"]
mod tests;
