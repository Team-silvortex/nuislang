use yir_core::{Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

pub fn parse_module(input: &str) -> Result<YirModule, String> {
    let mut module = YirModule::new("0.1");

    for (index, raw_line) in input.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        match tokens.first().copied() {
            Some("yir") => parse_header(&mut module, &tokens, line_no)?,
            Some("resource") => parse_resource(&mut module, &tokens, line_no)?,
            Some("edge") => parse_edge(&mut module, &tokens, line_no)?,
            Some(opcode) => parse_node(&mut module, opcode, &tokens, line_no)?,
            None => {}
        }
    }

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
        return Err(format!(
            "line {line_no}: expected `resource <name> <kind>`"
        ));
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

    let op = Operation::parse(
        opcode,
        tokens[3..].iter().map(|token| (*token).to_owned()).collect(),
    )
    .map_err(|error| format!("line {line_no}: {error}"))?;

    module.nodes.push(Node {
        name: tokens[1].to_owned(),
        resource: tokens[2].to_owned(),
        op,
    });
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
