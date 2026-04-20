use std::collections::BTreeMap;

use yir_core::{Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

pub fn parse_module(input: &str) -> Result<YirModule, String> {
    let mut module = YirModule::new("0.1");

    for (index, raw_line) in input.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let tokens = tokenize_line(line).map_err(|error| format!("line {line_no}: {error}"))?;
        match tokens.first().copied() {
            Some("yir") => parse_header(&mut module, &tokens, line_no)?,
            Some("resource") => parse_resource(&mut module, &tokens, line_no)?,
            Some("edge") => parse_edge(&mut module, &tokens, line_no)?,
            Some("node") => parse_shorthand_node(&mut module, &tokens, line_no)?,
            Some(opcode) => parse_node(&mut module, opcode, &tokens, line_no)?,
            None => {}
        }
    }

    ensure_implicit_cpu_nil_node(&mut module);
    synthesize_dependency_edges(&mut module);
    synthesize_lane_effect_edges(&mut module);

    Ok(module)
}

fn parse_header(module: &mut YirModule, tokens: &[&str], line_no: usize) -> Result<(), String> {
    if tokens.len() != 2 {
        return Err(format!("line {line_no}: expected `yir <version>`"));
    }

    module.version = tokens[1].to_owned();
    Ok(())
}

fn parse_resource(module: &mut YirModule, tokens: &[&str], line_no: usize) -> Result<(), String> {
    if tokens.len() != 3 {
        return Err(format!("line {line_no}: expected `resource <name> <kind>`"));
    }

    module.resources.push(Resource {
        name: tokens[1].to_owned(),
        kind: ResourceKind::parse(tokens[2]),
    });
    Ok(())
}

fn parse_node(
    module: &mut YirModule,
    opcode: &str,
    tokens: &[&str],
    line_no: usize,
) -> Result<(), String> {
    if tokens.len() < 3 {
        return Err(format!(
            "line {line_no}: expected `{opcode} <name> <resource> [args...]`"
        ));
    }

    let (resource_name, lane) = split_resource_lane(tokens[2]);
    let op = Operation::parse(
        opcode,
        tokens[3..]
            .iter()
            .map(|token| (*token).to_owned())
            .collect(),
    )
    .map_err(|error| format!("line {line_no}: {error}"))?;

    module.nodes.push(Node {
        name: tokens[1].to_owned(),
        resource: resource_name.to_owned(),
        op,
    });
    if let Some(lane) = lane {
        module
            .node_lanes
            .insert(tokens[1].to_owned(), lane.to_owned());
    }
    Ok(())
}

fn parse_shorthand_node(
    module: &mut YirModule,
    tokens: &[&str],
    line_no: usize,
) -> Result<(), String> {
    if tokens.len() < 4 {
        return Err(format!(
            "line {line_no}: expected `node <instr> <name> <resource> [args...]`"
        ));
    }

    let instruction = tokens[1];
    let name = tokens[2];
    let resource = tokens[3];
    let args = tokens[4..]
        .iter()
        .map(|token| (*token).to_owned())
        .collect::<Vec<_>>();
    let (resource_name, lane) = split_resource_lane(resource);
    let opcode = canonicalize_shorthand_opcode(module, instruction, name, resource, &args)
        .map_err(|error| format!("line {line_no}: {error}"))?;

    let op = Operation::parse(&opcode, args).map_err(|error| format!("line {line_no}: {error}"))?;
    module.nodes.push(Node {
        name: name.to_owned(),
        resource: resource_name.to_owned(),
        op,
    });
    if let Some(lane) = lane {
        module.node_lanes.insert(name.to_owned(), lane.to_owned());
    }
    Ok(())
}

fn parse_edge(module: &mut YirModule, tokens: &[&str], line_no: usize) -> Result<(), String> {
    if tokens.len() != 4 {
        return Err(format!(
            "line {line_no}: expected `edge <kind> <from> <to>`"
        ));
    }

    module.edges.push(Edge {
        kind: EdgeKind::parse(tokens[1]).map_err(|error| format!("line {line_no}: {error}"))?,
        from: tokens[2].to_owned(),
        to: tokens[3].to_owned(),
    });
    Ok(())
}

fn tokenize_line(line: &str) -> Result<Vec<&str>, String> {
    let mut tokens = Vec::new();
    let mut start = None::<usize>;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in line.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => {
                    let token_start = start
                        .take()
                        .ok_or_else(|| "internal tokenizer error".to_owned())?;
                    tokens.push(&line[token_start..index]);
                    in_string = false;
                }
                _ => {}
            }
            continue;
        }

        if ch.is_whitespace() {
            if let Some(token_start) = start.take() {
                tokens.push(&line[token_start..index]);
            }
            continue;
        }

        if ch == '"' {
            if start.is_some() {
                return Err("unexpected quote inside token".to_owned());
            }
            in_string = true;
            start = Some(index + ch.len_utf8());
            continue;
        }

        if start.is_none() {
            start = Some(index);
        }
    }

    if in_string {
        return Err("unterminated string literal".to_owned());
    }

    if let Some(token_start) = start.take() {
        tokens.push(&line[token_start..]);
    }

    Ok(tokens)
}

fn canonicalize_shorthand_opcode(
    module: &YirModule,
    instruction: &str,
    name: &str,
    resource: &str,
    args: &[String],
) -> Result<String, String> {
    let family = module
        .resources
        .iter()
        .find(|candidate| candidate.name == split_resource_lane(resource).0)
        .map(|resource| resource.kind.family().to_owned())
        .ok_or_else(|| format!("shorthand node references unknown resource `{resource}`"))?;

    let opcode = match family.as_str() {
        "cpu" => canonicalize_cpu_shorthand(instruction, name, args)?,
        "data" => canonicalize_data_shorthand(instruction)?,
        "shader" => canonicalize_domain_passthrough("shader", instruction),
        "kernel" => canonicalize_domain_passthrough("kernel", instruction),
        other => canonicalize_domain_passthrough(other, instruction),
    };
    Ok(opcode)
}

fn canonicalize_cpu_shorthand(
    instruction: &str,
    name: &str,
    args: &[String],
) -> Result<String, String> {
    let opcode = match instruction {
        "const" => cpu_const_opcode(args),
        "const.bool" => "cpu.const_bool".to_owned(),
        "const.i32" => "cpu.const_i32".to_owned(),
        "const.i64" => "cpu.const_i64".to_owned(),
        "const.f32" => "cpu.const_f32".to_owned(),
        "const.f64" => "cpu.const_f64".to_owned(),
        "alloc" => "cpu.alloc_node".to_owned(),
        "alloc.node" => "cpu.alloc_node".to_owned(),
        "alloc.buffer" => "cpu.alloc_buffer".to_owned(),
        "borrow" => "cpu.borrow".to_owned(),
        "borrow_end" => "cpu.borrow_end".to_owned(),
        "move" => "cpu.move_ptr".to_owned(),
        "move.ptr" => "cpu.move_ptr".to_owned(),
        "load" => {
            if name.eq_ignore_ascii_case("next") || name.contains("next") {
                "cpu.load_next".to_owned()
            } else {
                "cpu.load_value".to_owned()
            }
        }
        "load.value" => "cpu.load_value".to_owned(),
        "load.next" => "cpu.load_next".to_owned(),
        "load.len" => "cpu.buffer_len".to_owned(),
        "load_at" => "cpu.load_at".to_owned(),
        "store" => "cpu.store_value".to_owned(),
        "store.value" => "cpu.store_value".to_owned(),
        "store.next" => "cpu.store_next".to_owned(),
        "store_at" => "cpu.store_at".to_owned(),
        "free" => "cpu.free".to_owned(),
        "print" => "cpu.print".to_owned(),
        "null" => "cpu.null".to_owned(),
        "add" | "sub" | "mul" | "div" | "rem" | "eq" | "ne" | "lt" | "gt" | "le" | "ge" | "and"
        | "or" | "xor" | "shl" | "shr" | "neg" | "not" | "select" => {
            format!("cpu.{instruction}")
        }
        other => return Ok(format!("cpu.{other}")),
    };
    Ok(opcode)
}

fn canonicalize_data_shorthand(instruction: &str) -> Result<String, String> {
    let opcode = match instruction {
        "move" => "data.move",
        "copy_window" => "data.copy_window",
        "immutable_window" => "data.immutable_window",
        "marker" => "data.marker",
        "output_pipe" => "data.output_pipe",
        "input_pipe" => "data.input_pipe",
        "handle_table" => "data.handle_table",
        "bind_core" => "data.bind_core",
        other => return Ok(format!("data.{other}")),
    };
    Ok(opcode.to_owned())
}

fn canonicalize_domain_passthrough(domain: &str, instruction: &str) -> String {
    format!("{domain}.{instruction}")
}

fn cpu_const_opcode(args: &[String]) -> String {
    match args.first().map(String::as_str) {
        Some("true" | "false") => "cpu.const_bool".to_owned(),
        Some(raw) if raw.parse::<i64>().is_ok() => "cpu.const_i64".to_owned(),
        Some(raw) if raw.parse::<f64>().is_ok() && raw.contains('.') => "cpu.const_f64".to_owned(),
        _ => "cpu.const".to_owned(),
    }
}

fn synthesize_dependency_edges(module: &mut YirModule) {
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_resources = module
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node.resource.clone()))
        .collect::<BTreeMap<_, _>>();
    let node_names = module
        .nodes
        .iter()
        .map(|node| node.name.clone())
        .collect::<std::collections::BTreeSet<_>>();

    for node in &module.nodes {
        for arg in &node.op.args {
            if !node_names.contains(arg) {
                continue;
            }
            let from_family = node_resources
                .get(arg)
                .and_then(|resource| resource_families.get(resource));
            let to_family = resource_families.get(&node.resource);
            let kind = if from_family.is_some() && from_family == to_family {
                EdgeKind::Dep
            } else {
                EdgeKind::CrossDomainExchange
            };
            let exists = module
                .edges
                .iter()
                .any(|edge| edge.kind == kind && edge.from == *arg && edge.to == node.name);
            if !exists {
                module.edges.push(Edge {
                    kind,
                    from: arg.clone(),
                    to: node.name.clone(),
                });
            }
        }
    }
}

fn synthesize_lane_effect_edges(module: &mut YirModule) {
    let mut previous_by_queue = BTreeMap::<String, String>::new();

    for node in &module.nodes {
        let Some(lane) = module.node_lanes.get(&node.name) else {
            continue;
        };
        let queue = format!("{}@{}", node.resource, lane);
        if let Some(previous) = previous_by_queue.get(&queue) {
            let exists = module.edges.iter().any(|edge| {
                edge.kind == EdgeKind::Effect && edge.from == *previous && edge.to == node.name
            });
            if !exists {
                module.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: previous.clone(),
                    to: node.name.clone(),
                });
            }
        }
        previous_by_queue.insert(queue, node.name.clone());
    }
}

fn ensure_implicit_cpu_nil_node(module: &mut YirModule) {
    if module.nodes.iter().any(|node| node.name == "nil") {
        return;
    }
    let uses_nil = module
        .nodes
        .iter()
        .any(|node| node.op.args.iter().any(|arg| arg == "nil"));
    if !uses_nil {
        return;
    }
    let Some(resource) = module
        .resources
        .iter()
        .find(|resource| resource.kind.family() == "cpu")
    else {
        return;
    };
    module.nodes.push(Node {
        name: "nil".to_owned(),
        resource: resource.name.clone(),
        op: Operation::parse("cpu.null", Vec::new()).expect("cpu.null is valid"),
    });
}

fn split_resource_lane(raw: &str) -> (&str, Option<&str>) {
    match raw.split_once('@') {
        Some((resource, lane)) if !resource.is_empty() && !lane.is_empty() => {
            (resource, Some(lane))
        }
        _ => (raw, None),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_module;
    use yir_core::EdgeKind;

    #[test]
    fn parses_shorthand_cpu_nodes_and_infers_dep_edges() {
        let module = parse_module(
            r#"
resource cpu0 cpu.arm64

node const tail_value cpu0 30
node alloc tail cpu0 tail_value nil
node const head_value cpu0 10
node alloc head cpu0 head_value tail
node borrow head_ref cpu0 head
node load head_val cpu0 head_ref
node load next cpu0 head_ref
node borrow tail_ref cpu0 next
node load tail_val cpu0 tail_ref
node add sum cpu0 head_val tail_val
node print out cpu0 sum
"#,
        )
        .unwrap();

        assert!(module
            .nodes
            .iter()
            .any(|node| node.name == "tail_value" && node.op.full_name() == "cpu.const_i64"));
        assert!(module
            .nodes
            .iter()
            .any(|node| node.name == "tail" && node.op.full_name() == "cpu.alloc_node"));
        assert!(module
            .nodes
            .iter()
            .any(|node| node.name == "next" && node.op.full_name() == "cpu.load_next"));
        assert!(module.edges.iter().any(|edge| edge.kind == EdgeKind::Dep
            && edge.from == "tail_value"
            && edge.to == "tail"));
        assert!(module.edges.iter().any(|edge| edge.kind == EdgeKind::Dep
            && edge.from == "tail_ref"
            && edge.to == "tail_val"));
    }

    #[test]
    fn infers_xfer_for_cross_domain_args() {
        let module = parse_module(
            r#"
resource cpu0 cpu.arm64
resource fabric0 data.fabric

node const seed cpu0 7
node output_pipe packet fabric0 seed
"#,
        )
        .unwrap();

        assert!(module
            .edges
            .iter()
            .any(|edge| edge.kind == EdgeKind::CrossDomainExchange
                && edge.from == "seed"
                && edge.to == "packet"));
    }

    #[test]
    fn parses_stable_typed_shorthand_without_heuristics() {
        let module = parse_module(
            r#"
resource cpu0 cpu.arm64

node const.i64 tail_value cpu0 30
node alloc.node tail cpu0 tail_value nil
node const.i64 head_value cpu0 10
node alloc.node head cpu0 head_value tail
node borrow head_ref cpu0 head
node load.value head_val cpu0 head_ref
node load.next next_ptr cpu0 head_ref
node borrow tail_ref cpu0 next_ptr
node load.value tail_val cpu0 tail_ref
node add sum cpu0 head_val tail_val
node print out cpu0 sum
"#,
        )
        .unwrap();

        assert!(module
            .nodes
            .iter()
            .any(|node| node.name == "tail_value" && node.op.full_name() == "cpu.const_i64"));
        assert!(module
            .nodes
            .iter()
            .any(|node| node.name == "tail" && node.op.full_name() == "cpu.alloc_node"));
        assert!(module
            .nodes
            .iter()
            .any(|node| node.name == "next_ptr" && node.op.full_name() == "cpu.load_next"));
    }

    #[test]
    fn parses_optional_lane_suffix_on_resource_token() {
        let module = parse_module(
            r#"
resource cpu0 cpu.arm64

node const.i64 seed cpu0@mem 7
node print out cpu0@main seed
"#,
        )
        .unwrap();

        assert_eq!(
            module.node_lanes.get("seed").map(String::as_str),
            Some("mem")
        );
        assert_eq!(
            module.node_lanes.get("out").map(String::as_str),
            Some("main")
        );
        assert!(module
            .edges
            .iter()
            .any(|edge| edge.kind == EdgeKind::Dep && edge.from == "seed" && edge.to == "out"));
    }

    #[test]
    fn synthesizes_serial_effect_edges_within_same_resource_lane() {
        let module = parse_module(
            r#"
resource cpu0 cpu.arm64

node const.i64 a cpu0@mem 1
node const.i64 b cpu0@mem 2
node add sum cpu0@main a b
node print out cpu0@main sum
"#,
        )
        .unwrap();

        assert!(module
            .edges
            .iter()
            .any(|edge| edge.kind == EdgeKind::Effect && edge.from == "a" && edge.to == "b"));
        assert!(module
            .edges
            .iter()
            .any(|edge| edge.kind == EdgeKind::Effect && edge.from == "sum" && edge.to == "out"));
    }
}
