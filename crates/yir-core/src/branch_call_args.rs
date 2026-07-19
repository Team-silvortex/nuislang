#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BranchOwnedCallArgs<'a> {
    pub condition: &'a str,
    pub then_callee: &'a str,
    pub else_callee: &'a str,
    pub owner: &'a str,
    pub then_scalar_args: &'a [String],
    pub else_scalar_args: &'a [String],
}

pub fn parse_branch_owned_call_args(args: &[String]) -> Option<BranchOwnedCallArgs<'_>> {
    let then_count = args.get(4)?.parse::<usize>().ok()?;
    let then_start = 5usize;
    let then_end = then_start.checked_add(then_count)?;
    let then_scalar_args = args.get(then_start..then_end)?;
    let else_count = args.get(then_end)?.parse::<usize>().ok()?;
    let else_start = then_end.checked_add(1)?;
    let else_end = else_start.checked_add(else_count)?;
    if else_end != args.len() {
        return None;
    }
    Some(BranchOwnedCallArgs {
        condition: args.first()?,
        then_callee: args.get(1)?,
        else_callee: args.get(2)?,
        owner: args.get(3)?,
        then_scalar_args,
        else_scalar_args: &args[else_start..else_end],
    })
}
