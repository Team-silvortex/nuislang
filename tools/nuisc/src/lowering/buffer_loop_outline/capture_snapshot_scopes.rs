use super::*;

pub(super) struct Scope<'a> {
    pub body: &'a [NirStmt],
    pub in_loop: bool,
    pub returns: bool,
}

enum Step {
    Return,
    Escape,
    Branch(usize, usize),
}

#[derive(Clone, Copy)]
struct Flow {
    falls_through: bool,
    escapes: bool,
}

// Assign lexical scopes once, then prove their exits bottom-up. A child loop
// may execute zero times; its returns cannot close the containing scope.
pub(super) fn collect(body: &[NirStmt]) -> Vec<Scope<'_>> {
    let mut scopes = vec![Scope {
        body,
        in_loop: false,
        returns: false,
    }];
    let mut steps = Vec::new();
    let mut index = 0;
    while index < scopes.len() {
        let body = scopes[index].body;
        let in_loop = scopes[index].in_loop;
        let mut flow = Vec::new();
        for stmt in body {
            match stmt {
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    let child = scopes.len();
                    scopes.push(Scope {
                        body: then_body,
                        in_loop,
                        returns: false,
                    });
                    scopes.push(Scope {
                        body: else_body,
                        in_loop,
                        returns: false,
                    });
                    flow.push(Step::Branch(child, child + 1));
                }
                NirStmt::While { body, .. } => {
                    scopes.push(Scope {
                        body,
                        in_loop: true,
                        returns: false,
                    });
                }
                NirStmt::Return(_) => flow.push(Step::Return),
                NirStmt::Break | NirStmt::Continue => flow.push(Step::Escape),
                _ => {}
            }
        }
        steps.push(flow);
        index += 1;
    }
    let initial = Flow {
        falls_through: true,
        escapes: false,
    };
    let mut flows = vec![initial; scopes.len()];
    for index in (0..scopes.len()).rev() {
        let mut flow = initial;
        for step in &steps[index] {
            if !flow.falls_through {
                break;
            }
            match step {
                Step::Return => flow.falls_through = false,
                Step::Escape => {
                    flow.falls_through = false;
                    flow.escapes = true;
                }
                Step::Branch(left, right) => {
                    flow.escapes |= flows[*left].escapes || flows[*right].escapes;
                    flow.falls_through = flows[*left].falls_through || flows[*right].falls_through;
                }
            }
        }
        flows[index] = flow;
        scopes[index].returns = !flow.falls_through && !flow.escapes;
    }
    scopes
}
