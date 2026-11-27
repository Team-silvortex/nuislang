//! Synchronous observation of native Data effects. Only owned words and a
//! bounded site identity cross the callback; the bound YIR supplies the schema.
use std::{cell::RefCell, collections::BTreeMap};
use yir_core::{
    native_scalar_session::{MAX_PIPE_SITE_BYTES, MAX_SCALAR_SLOTS},
    DataPipe, DataPipeDirection, DataWindow, Node, StructValue, Value, YirModule,
};
use yir_exec::ExecutionTrace;

pub(super) struct Event(String, Vec<i64>);
#[derive(Default)]
struct Batch {
    events: Vec<Event>,
    overflow: bool,
}
thread_local! { static ACTIVE: RefCell<Option<Batch>> = const { RefCell::new(None) }; }

pub(super) fn capture<T>(f: impl FnOnce() -> T) -> Result<(T, Vec<Event>), String> {
    struct Restore(Option<Batch>);
    impl Drop for Restore {
        fn drop(&mut self) {
            ACTIVE.replace(self.0.take());
        }
    }
    let _scope = Restore(ACTIVE.replace(Some(Batch::default())));
    let value = f();
    let batch = ACTIVE.take().expect("active native callback");
    if batch.overflow {
        return Err("native Data pipe event limit exceeded".into());
    }
    Ok((value, batch.events))
}

/// # Safety
/// The static producer supplies readable, nonoverlapping UTF-8 name bytes and
/// aligned i64 words for the stated lengths. Neither pointer is retained.
#[no_mangle]
pub unsafe extern "C" fn nuis_native_data_pipe_event_v1(
    site: *const u8,
    len: u64,
    words: *const i64,
    count: u64,
) -> i32 {
    if site.is_null()
        || words.is_null()
        || !(1..=MAX_PIPE_SITE_BYTES as u64).contains(&len)
        || !(1..=MAX_SCALAR_SLOTS as u64).contains(&count)
    {
        return 1;
    }
    ACTIVE.with_borrow_mut(|active| {
        let Some(batch) = active else {
            return 1;
        };
        if batch.events.len() >= 4096 {
            batch.overflow = true;
            return 0;
        }
        let Ok(name) =
            std::str::from_utf8(unsafe { std::slice::from_raw_parts(site, len as usize) })
        else {
            return 1;
        };
        batch.events.push(Event(
            name.to_owned(),
            unsafe { std::slice::from_raw_parts(words, count as usize) }.to_vec(),
        ));
        0
    })
}

pub(super) struct SiteIndex<'a> {
    module: &'a YirModule,
    nodes: BTreeMap<&'a str, &'a Node>,
}

impl<'a> SiteIndex<'a> {
    pub fn new(module: &'a YirModule) -> Self {
        Self {
            module,
            nodes: module.nodes.iter().map(|n| (n.name.as_str(), n)).collect(),
        }
    }

    fn source(&self, n: &Node, op: &str) -> Result<&Node, String> {
        n.op.args
            .first()
            .and_then(|name| self.nodes.get(name.as_str()))
            .copied()
            .filter(|p| p.op.full_name() == op)
            .ok_or_else(|| format!("native pipe site `{}` requires a {op} producer", n.name))
    }

    pub fn append(&self, events: Vec<Event>, trace: &mut ExecutionTrace) -> Result<(), String> {
        let mut text_bytes = 0usize;
        for Event(name, words) in events {
            let site = self
                .nodes
                .get(name.as_str())
                .ok_or("unknown native pipe site")?;
            let output = match site.op.full_name().as_str() {
                "data.output_pipe" => true,
                "data.input_pipe" => false,
                _ => return Err("not a native Data pipe site".into()),
            };
            let out = if output {
                *site
            } else {
                self.source(site, "data.output_pipe")?
            };
            let packet = self.source(self.source(out, "data.immutable_window")?, "cpu.struct")?;
            let (ty, fields) = packet
                .op
                .args
                .split_first()
                .ok_or("native Data pipe packet has no type")?;
            if fields.len() != words.len() {
                return Err("native Data pipe packet width mismatch".into());
            }
            let fields = fields
                .iter()
                .zip(words)
                .map(|(binding, word)| {
                    let (field, _) = binding
                        .split_once('=')
                        .ok_or("invalid native Data pipe field")?;
                    Ok((field.trim().to_owned(), Value::Int(word)))
                })
                .collect::<Result<_, String>>()?;
            let window = Value::DataWindow(DataWindow {
                base: Box::new(Value::Struct(StructValue {
                    type_name: ty.clone(),
                    fields,
                })),
                offset: 0,
                len: 1,
                immutable: true,
            });
            let value = if output {
                Value::DataPipe(DataPipe {
                    direction: DataPipeDirection::Output,
                    payload: Box::new(window),
                })
            } else {
                window
            };
            let resource = self
                .module
                .resources
                .iter()
                .find(|r| r.name == site.resource)
                .ok_or("missing native Data pipe resource")?;
            let lane = self
                .module
                .node_lanes
                .get(&name)
                .ok_or("missing native Data pipe lane")?;
            let event = yir_core::data_pipe_event(resource, output, &value);
            text_bytes += event.len();
            if text_bytes > 4 * 1024 * 1024 {
                return Err("native Data pipe trace byte limit exceeded".into());
            }
            trace
                .lane_events
                .entry(format!("{}@{lane}", resource.name))
                .or_default()
                .push(event.clone());
            trace.events.push(event);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emit(name: &str, words: &[i64]) -> i32 {
        unsafe {
            nuis_native_data_pipe_event_v1(
                name.as_ptr(),
                name.len() as u64,
                words.as_ptr(),
                words.len() as u64,
            )
        }
    }

    #[test]
    fn captures_only_live_calls_in_order_and_isolates_scopes() {
        assert_ne!(emit("output", &[1]), 0);
        let (_, outer) = capture(|| {
            let mut packet = [-7, 0, 42];
            assert_eq!(emit("output", &packet), 0);
            let (_, inner) = capture(|| assert_eq!(emit("inner", &[17]), 0)).unwrap();
            assert_eq!(inner[0].0, "inner");
            assert_eq!(emit("input", &packet), 0);
            packet.fill(9);
        })
        .unwrap();
        assert_eq!(outer.len(), 2);
        assert_eq!((&outer[0].0[..], &outer[1].0[..]), ("output", "input"));
        assert_eq!(outer[0].1, [-7, 0, 42]);
        assert_eq!(outer[1].1, outer[0].1);
        assert_ne!(emit("input", &[1]), 0);
    }
}
