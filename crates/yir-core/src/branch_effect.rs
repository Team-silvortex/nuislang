#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchEffectAccess {
    ValueRead,
    ResourceRead,
    ResourceOwn,
}

impl BranchEffectAccess {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "value_read" => Self::ValueRead,
            "resource_read" => Self::ResourceRead,
            "resource_own" => Self::ResourceOwn,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ValueRead => "value_read",
            Self::ResourceRead => "resource_read",
            Self::ResourceOwn => "resource_own",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchEffectResult {
    Unit,
    I64,
    OwnedPointer,
}

impl BranchEffectResult {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "unit" => Self::Unit,
            "i64" => Self::I64,
            "owned_ptr" => Self::OwnedPointer,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::I64 => "i64",
            Self::OwnedPointer => "owned_ptr",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BranchEffectActionCapability {
    pub module: &'static str,
    pub instruction: &'static str,
    pub result: BranchEffectResult,
    pub operand_accesses: &'static [BranchEffectAccess],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedBranchEffectOperand {
    pub access: BranchEffectAccess,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedBranchEffectAction {
    pub module: String,
    pub instruction: String,
    pub result: BranchEffectResult,
    pub operands: Vec<PlannedBranchEffectOperand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchEffectOperand<'a> {
    pub access: BranchEffectAccess,
    pub value: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchEffectAction<'a> {
    pub module: &'a str,
    pub instruction: &'a str,
    pub result: BranchEffectResult,
    pub operands: Vec<BranchEffectOperand<'a>>,
}

impl BranchEffectAction<'_> {
    pub fn matches_capability(&self, capability: &BranchEffectActionCapability) -> bool {
        self.module == capability.module
            && self.instruction == capability.instruction
            && self.result == capability.result
            && self
                .operands
                .iter()
                .map(|operand| operand.access)
                .eq(capability.operand_accesses.iter().copied())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchEffectArgs<'a> {
    pub condition: &'a str,
    pub merge_result: BranchEffectResult,
    pub address_kind: Option<&'a str>,
    pub nullable: bool,
    pub then_actions: Vec<BranchEffectAction<'a>>,
    pub else_actions: Vec<BranchEffectAction<'a>>,
}

pub fn parse_branch_effect_args(args: &[String]) -> Option<BranchEffectArgs<'_>> {
    let condition = args.first()?.as_str();
    let merge_result = BranchEffectResult::parse(args.get(1)?)?;
    let mut cursor = 2usize;
    let address_kind = args
        .get(cursor)
        .and_then(|value| value.strip_prefix("address_kind="));
    let nullable = if address_kind.is_some() {
        cursor += 1;
        let value = args.get(cursor)?.strip_prefix("nullable=")?;
        cursor += 1;
        match value {
            "true" => true,
            "false" => false,
            _ => return None,
        }
    } else {
        false
    };
    let then_actions = parse_actions(args, &mut cursor)?;
    let else_actions = parse_actions(args, &mut cursor)?;
    (cursor == args.len()).then_some(BranchEffectArgs {
        condition,
        merge_result,
        address_kind,
        nullable,
        then_actions,
        else_actions,
    })
}

pub fn branch_effect_merge_is_valid(args: &BranchEffectArgs<'_>) -> bool {
    let metadata_valid = match args.merge_result {
        BranchEffectResult::OwnedPointer => args.address_kind.is_none_or(|kind| {
            !kind.is_empty()
                && kind
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        }),
        _ => args.address_kind.is_none() && !args.nullable,
    };
    metadata_valid
        && (args.merge_result == BranchEffectResult::Unit
            || [args.then_actions.last(), args.else_actions.last()]
                .into_iter()
                .all(|action| action.is_some_and(|action| action.result == args.merge_result)))
}

fn parse_actions<'a>(
    args: &'a [String],
    cursor: &mut usize,
) -> Option<Vec<BranchEffectAction<'a>>> {
    let count = args.get(*cursor)?.parse::<usize>().ok()?;
    *cursor += 1;
    let mut actions = Vec::with_capacity(count);
    for _ in 0..count {
        let module = args.get(*cursor)?.as_str();
        let instruction = args.get(cursor.checked_add(1)?)?.as_str();
        let result = BranchEffectResult::parse(args.get(cursor.checked_add(2)?)?)?;
        let arity = args.get(cursor.checked_add(3)?)?.parse::<usize>().ok()?;
        *cursor = cursor.checked_add(4)?;
        let mut operands = Vec::with_capacity(arity);
        for _ in 0..arity {
            let access = BranchEffectAccess::parse(args.get(*cursor)?)?;
            let value = args.get(cursor.checked_add(1)?)?.as_str();
            *cursor = cursor.checked_add(2)?;
            operands.push(BranchEffectOperand { access, value });
        }
        actions.push(BranchEffectAction {
            module,
            instruction,
            result,
            operands,
        });
    }
    Some(actions)
}

pub fn branch_effect_inputs<'a>(args: &'a BranchEffectArgs<'a>) -> Vec<&'a str> {
    let mut inputs = vec![args.condition];
    for action in args.then_actions.iter().chain(&args.else_actions) {
        inputs.extend(action.operands.iter().map(|operand| operand.value));
    }
    inputs
}

impl crate::ModRegistry {
    pub fn describe_branch_effect_node(
        &self,
        node: &crate::Node,
    ) -> Result<Option<crate::InstructionSemantics>, String> {
        if node.op.instruction != "branch_effect" {
            return Ok(None);
        }
        let args = parse_branch_effect_args(&node.op.args)
            .ok_or_else(|| format!("node `{}` has invalid branch effect arguments", node.name))?;
        if !branch_effect_merge_is_valid(&args) {
            return Err(format!(
                "node `{}` branch actions do not produce the declared {:?} merge result",
                node.name, args.merge_result
            ));
        }
        for action in args.then_actions.iter().chain(&args.else_actions) {
            let capability = self
                .branch_effect_action_capability(action.module, action.instruction)
                .filter(|capability| action.matches_capability(capability))
                .ok_or_else(|| {
                    format!(
                        "node `{}` references branch action `{}.{}` with an undeclared result or operand contract",
                        node.name, action.module, action.instruction
                    )
                })?;
            debug_assert_eq!(capability.module, action.module);
        }
        Ok(Some(crate::InstructionSemantics::effect(
            branch_effect_inputs(&args)
                .into_iter()
                .map(str::to_owned)
                .collect(),
        )))
    }

    pub fn execute_branch_effect_node(
        &self,
        node: &crate::Node,
        resource: &crate::Resource,
        state: &mut crate::ExecutionState,
    ) -> Result<Option<crate::Value>, String> {
        if self.describe_branch_effect_node(node)?.is_none() {
            return Ok(None);
        }
        let args = parse_branch_effect_args(&node.op.args).expect("validated branch effect");
        let select_then = match state.expect_value(args.condition)? {
            crate::Value::Bool(value) => *value,
            crate::Value::Int(value) => *value != 0,
            other => {
                return Err(format!(
                    "node `{}` expects bool or i64 branch condition, got {other}",
                    node.name
                ));
            }
        };
        let selected = if select_then {
            &args.then_actions
        } else {
            &args.else_actions
        };
        let mut selected_result = crate::Value::Unit;
        for action in selected {
            let owner = self.lookup(action.module).ok_or_else(|| {
                format!(
                    "node `{}` cannot dispatch unregistered branch action `{}.{}`",
                    node.name, action.module, action.instruction
                )
            })?;
            selected_result = owner.execute_branch_effect_action(action, node, resource, state)?;
        }
        match args.merge_result {
            BranchEffectResult::Unit => Ok(Some(crate::Value::Unit)),
            BranchEffectResult::I64 if matches!(selected_result, crate::Value::Int(_)) => {
                Ok(Some(selected_result))
            }
            BranchEffectResult::OwnedPointer
                if matches!(selected_result, crate::Value::Pointer(_)) =>
            {
                Ok(Some(selected_result))
            }
            result => Err(format!(
                "node `{}` selected branch did not produce its declared {result:?} result",
                node.name
            )),
        }
    }
}
