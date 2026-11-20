use super::*;
use nuis_semantics::model::{NirStmt, NirTypeRef};

pub(super) fn validate(
    module: &NirModule,
    shadows: &BTreeMap<String, BTreeMap<String, Shadow>>,
) -> Result<(), String> {
    for (function_name, shadows) in shadows {
        let function = module.functions.iter().find(|f| f.name == *function_name)
            .ok_or_else(|| format!("resource shadow function `{function_name}` disappeared during semantic lowering"))?;
        let visible = function
            .params
            .iter()
            .map(|p| (p.name.as_str(), &p.ty))
            .collect::<BTreeMap<_, _>>();
        let mut seen = BTreeSet::new();
        let mut pending = vec![(function.body.as_slice(), visible)];
        while let Some((body, mut visible)) = pending.pop() {
            for stmt in body {
                match stmt {
                    NirStmt::Let { name, ty, .. } => {
                        declaration(name, ty.as_ref(), &mut visible, shadows, &mut seen)?;
                    }
                    NirStmt::Const { name, ty, .. } => {
                        declaration(name, Some(ty), &mut visible, shadows, &mut seen)?;
                    }
                    NirStmt::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        pending.push((else_body.as_slice(), visible.clone()));
                        pending.push((then_body.as_slice(), visible.clone()));
                    }
                    NirStmt::While { body, .. } => pending.push((body.as_slice(), visible.clone())),
                    _ => {}
                }
            }
        }
        if seen.len() != shadows.len() {
            return Err(format!("resource shadow declarations disappeared from `{function_name}` before type admission"));
        }
    }
    Ok(())
}

fn declaration<'a>(
    name: &'a str,
    ty: Option<&'a NirTypeRef>,
    visible: &mut BTreeMap<&'a str, &'a NirTypeRef>,
    shadows: &BTreeMap<String, Shadow>,
    seen: &mut BTreeSet<&'a str>,
) -> Result<(), String> {
    if let Some(shadow) = shadows.get(name).filter(|_| !seen.contains(name)) {
        let previous = visible.get(shadow.previous.as_str()).copied();
        let outer_kind = scalar_kind(previous);
        if outer_kind.is_none() || outer_kind != scalar_kind(ty) {
            return Err(format!(
                "resource shadow `{}` requires matching resolved i64/bool kinds (outer={}, local={})",
                shadow.source,
                previous
                    .map(NirTypeRef::render)
                    .unwrap_or_else(|| "unresolved".into()),
                ty.map(NirTypeRef::render)
                    .unwrap_or_else(|| "unresolved".into()),
            ));
        }
        seen.insert(name);
    }
    if let Some(ty) = ty {
        visible.insert(name, ty);
    } else {
        visible.remove(name);
    }
    Ok(())
}

fn scalar_kind(ty: Option<&NirTypeRef>) -> Option<&str> {
    ty.filter(|ty| {
        matches!(ty.name.as_str(), "i64" | "bool")
            && ty.generic_args.is_empty()
            && !ty.is_ref
            && !ty.is_optional
    })
    .map(|ty| ty.name.as_str())
}
