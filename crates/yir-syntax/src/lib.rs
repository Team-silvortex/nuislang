use std::collections::BTreeMap;

use yir_core::{
    Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirFunction, YirFunctionParameter,
    YirFunctionResult, YirFunctionRole, YirModule, YirValueOwnership,
};

pub fn parse_module(input: &str) -> Result<YirModule, String> {
    let mut module = parse_explicit_module(input)?;

    ensure_implicit_cpu_nil_node(&mut module);
    synthesize_dependency_edges(&mut module);
    synthesize_lane_effect_edges(&mut module);

    Ok(module)
}

/// Parses only records present in the text, without inferring nodes or edges.
pub fn parse_explicit_module(input: &str) -> Result<YirModule, String> {
    let mut module = YirModule::new("0.1");

    for (index, raw_line) in input.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let tokens = tokenize_line(line).map_err(|error| format!("line {line_no}: {error}"))?;
        match tokens.first().map(String::as_str) {
            Some("yir") => parse_header(&mut module, &tokens, line_no)?,
            Some("resource") => parse_resource(&mut module, &tokens, line_no)?,
            Some("function") => parse_function(&mut module, &tokens, line_no)?,
            Some("function-param") => parse_function_parameter(&mut module, &tokens, line_no)?,
            Some("function-result") => parse_function_result(&mut module, &tokens, line_no)?,
            Some("function-node") => parse_function_node(&mut module, &tokens, line_no)?,
            Some("edge") => parse_edge(&mut module, &tokens, line_no)?,
            Some("node") => parse_shorthand_node(&mut module, &tokens, line_no)?,
            Some(opcode) => parse_node(&mut module, opcode, &tokens, line_no)?,
            None => {}
        }
    }

    Ok(module)
}

fn parse_function(module: &mut YirModule, tokens: &[String], line_no: usize) -> Result<(), String> {
    if tokens.len() != 4 {
        return Err(format!(
            "line {line_no}: expected `function <name> <domain> <entry|helper|provider>`"
        ));
    }
    module.functions.push(YirFunction {
        name: tokens[1].to_owned(),
        domain: tokens[2].to_owned(),
        role: YirFunctionRole::parse(&tokens[3])
            .map_err(|error| format!("line {line_no}: {error}"))?,
        parameters: Vec::new(),
        result: None,
        body_nodes: Vec::new(),
    });
    Ok(())
}

fn parse_function_parameter(
    module: &mut YirModule,
    tokens: &[String],
    line_no: usize,
) -> Result<(), String> {
    if tokens.len() != 6 {
        return Err(format!(
            "line {line_no}: expected `function-param <function> <name> <type> <value|borrowed|owned> <node>`"
        ));
    }
    let function = find_function_mut(module, &tokens[1], line_no)?;
    function.parameters.push(YirFunctionParameter {
        name: tokens[2].to_owned(),
        ty: tokens[3].to_owned(),
        ownership: YirValueOwnership::parse(&tokens[4])
            .map_err(|error| format!("line {line_no}: {error}"))?,
        node: tokens[5].to_owned(),
    });
    Ok(())
}

fn parse_function_result(
    module: &mut YirModule,
    tokens: &[String],
    line_no: usize,
) -> Result<(), String> {
    if tokens.len() != 5 {
        return Err(format!(
            "line {line_no}: expected `function-result <function> <type> <value|borrowed|owned> <node>`"
        ));
    }
    let function = find_function_mut(module, &tokens[1], line_no)?;
    if function.result.is_some() {
        return Err(format!(
            "line {line_no}: function `{}` has multiple result records",
            tokens[1]
        ));
    }
    function.result = Some(YirFunctionResult {
        ty: tokens[2].to_owned(),
        ownership: YirValueOwnership::parse(&tokens[3])
            .map_err(|error| format!("line {line_no}: {error}"))?,
        node: tokens[4].to_owned(),
    });
    Ok(())
}

fn parse_function_node(
    module: &mut YirModule,
    tokens: &[String],
    line_no: usize,
) -> Result<(), String> {
    if tokens.len() != 3 {
        return Err(format!(
            "line {line_no}: expected `function-node <function> <node>`"
        ));
    }
    find_function_mut(module, &tokens[1], line_no)?
        .body_nodes
        .push(tokens[2].to_owned());
    Ok(())
}

fn find_function_mut<'a>(
    module: &'a mut YirModule,
    name: &str,
    line_no: usize,
) -> Result<&'a mut YirFunction, String> {
    module
        .functions
        .iter_mut()
        .find(|function| function.name == name)
        .ok_or_else(|| {
            format!("line {line_no}: function metadata references unknown function `{name}`")
        })
}

fn parse_header(module: &mut YirModule, tokens: &[String], line_no: usize) -> Result<(), String> {
    if tokens.len() != 2 {
        return Err(format!("line {line_no}: expected `yir <version>`"));
    }

    module.version = tokens[1].to_owned();
    Ok(())
}

fn parse_resource(module: &mut YirModule, tokens: &[String], line_no: usize) -> Result<(), String> {
    if tokens.len() != 3 {
        return Err(format!("line {line_no}: expected `resource <name> <kind>`"));
    }

    module.resources.push(Resource {
        name: tokens[1].to_owned(),
        kind: ResourceKind::parse(&tokens[2]),
    });
    Ok(())
}

fn parse_node(
    module: &mut YirModule,
    opcode: &str,
    tokens: &[String],
    line_no: usize,
) -> Result<(), String> {
    if tokens.len() < 3 {
        return Err(format!(
            "line {line_no}: expected `{opcode} <name> <resource> [args...]`"
        ));
    }

    let (resource_name, lane) = split_resource_lane(&tokens[2]);
    let op = Operation::parse(opcode, tokens[3..].iter().cloned().collect())
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
    tokens: &[String],
    line_no: usize,
) -> Result<(), String> {
    if tokens.len() < 4 {
        return Err(format!(
            "line {line_no}: expected `node <instr> <name> <resource> [args...]`"
        ));
    }

    let instruction = &tokens[1];
    let name = &tokens[2];
    let resource = &tokens[3];
    let args = tokens[4..].iter().cloned().collect::<Vec<_>>();
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

fn parse_edge(module: &mut YirModule, tokens: &[String], line_no: usize) -> Result<(), String> {
    if tokens.len() != 4 {
        return Err(format!(
            "line {line_no}: expected `edge <kind> <from> <to>`"
        ));
    }

    module.edges.push(Edge {
        kind: EdgeKind::parse(&tokens[1]).map_err(|error| format!("line {line_no}: {error}"))?,
        from: tokens[2].to_owned(),
        to: tokens[3].to_owned(),
    });
    Ok(())
}

fn tokenize_line(line: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut token = None::<String>;
    let mut in_string = false;
    let mut quoted_token_closed = false;
    let mut chars = line.chars();

    while let Some(ch) = chars.next() {
        if in_string {
            match ch {
                '\\' => {
                    let escaped = chars
                        .next()
                        .ok_or_else(|| "unterminated string escape".to_owned())?;
                    let value = match escaped {
                        '\\' => '\\',
                        '"' => '"',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        other => return Err(format!("unsupported string escape `\\{other}`")),
                    };
                    token.as_mut().expect("quoted token is active").push(value);
                }
                '"' => {
                    in_string = false;
                    quoted_token_closed = true;
                }
                other => token.as_mut().expect("quoted token is active").push(other),
            }
            continue;
        }

        if ch.is_whitespace() {
            if let Some(value) = token.take() {
                tokens.push(value);
            }
            quoted_token_closed = false;
            continue;
        }

        if ch == '"' {
            if token.is_some() || quoted_token_closed {
                return Err("unexpected quote inside token".to_owned());
            }
            in_string = true;
            token = Some(String::new());
            continue;
        }

        if quoted_token_closed {
            return Err("expected whitespace after quoted token".to_owned());
        }
        token.get_or_insert_with(String::new).push(ch);
    }

    if in_string {
        return Err("unterminated string literal".to_owned());
    }

    if let Some(value) = token {
        tokens.push(value);
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
            let reverse_exists = module
                .edges
                .iter()
                .any(|edge| edge.from == node.name && edge.to == *previous);
            if !exists && !reverse_exists {
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
    use super::{parse_explicit_module, parse_module};
    use yir_core::EdgeKind;

    #[test]
    fn explicit_parser_does_not_synthesize_dependency_edges() {
        let source = r#"yir 0.1
resource cpu0 cpu.arm64
cpu.const_i64 left cpu0 1
cpu.add sum cpu0 left left
"#;

        let explicit = parse_explicit_module(source).expect("explicit module should parse");
        let inferred = parse_module(source).expect("inferred module should parse");

        assert!(explicit.edges.is_empty());
        assert_eq!(inferred.edges.len(), 1);
        assert_eq!(inferred.edges[0].kind, EdgeKind::Dep);
    }

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
    fn parses_typed_function_boundaries_and_body_membership() {
        let module = parse_module(
            r#"
resource cpu0 cpu.arm64

function main cpu entry
function-param main input i64 value input_node
function-result main i64 value output_node
function-node main input_node
function-node main output_node

cpu.param_i64 input_node cpu0 0
cpu.add output_node cpu0 input_node input_node
"#,
        )
        .unwrap();

        let function = &module.functions[0];
        assert_eq!(function.name, "main");
        assert_eq!(function.role.as_str(), "entry");
        assert_eq!(function.parameters[0].name, "input");
        assert_eq!(function.parameters[0].ownership.as_str(), "value");
        assert_eq!(
            function.result.as_ref().map(|result| result.node.as_str()),
            Some("output_node")
        );
        assert_eq!(function.body_nodes, ["input_node", "output_node"]);
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

    #[test]
    fn does_not_synthesize_lane_effect_edge_against_reverse_explicit_dependency() {
        let module = parse_module(
            r#"
resource cpu0 cpu.arm64

cpu.target_config lowering_cpu_target_config cpu0@contract arm64 cpu.arm64.apple_aapcs64 128
cpu.text lowering_cpu_target_contract_type cpu0@contract arch=symbol:arm64;abi=symbol:cpu.arm64.apple_aapcs64;vector_bits=i64:128
edge dep lowering_cpu_target_contract_type lowering_cpu_target_config
"#,
        )
        .unwrap();

        assert!(module.edges.iter().any(|edge| {
            edge.kind == EdgeKind::Dep
                && edge.from == "lowering_cpu_target_contract_type"
                && edge.to == "lowering_cpu_target_config"
        }));
        assert!(!module.edges.iter().any(|edge| {
            edge.kind == EdgeKind::Effect
                && edge.from == "lowering_cpu_target_config"
                && edge.to == "lowering_cpu_target_contract_type"
        }));
    }
}
