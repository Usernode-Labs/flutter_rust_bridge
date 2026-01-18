use crate::codegen::ir::mir::func::{MirFunc, MirFuncInput, OwnershipMode};
use crate::codegen::ir::mir::ty::delegate::MirTypeDelegate;
use crate::codegen::ir::mir::ty::MirType;
use convert_case::{Case, Casing};
use itertools::Itertools;

pub(crate) fn generate_code_inner_decode(func: &MirFunc) -> String {
    let interest_fields = filter_interest_fields(func);
    if interest_fields.is_empty() {
        return "".to_owned();
    }

    let declarations = (interest_fields.iter())
        .map(|info| {
            let field = &func.inputs[info.field_index];
            format!(
                "let mut api_{name}_guard = None;\n",
                name = get_variable_name(field)
            )
        })
        .join("");

    let var_orders = (interest_fields.iter().enumerate())
        .map(|(index, info)| {
            let field = &func.inputs[info.field_index];
            let mutable = (info.ownership_mode == OwnershipMode::RefMut).to_string();
            format!(
                "flutter_rust_bridge::for_generated::LockableOrderInfo::new(&api_{name}, {index}, {mutable})",
                name = get_variable_name(field)
            )
        })
        .join(", ");

    let match_arms = (interest_fields.iter().enumerate())
        .map(|(index, info)| {
            format!(
                "{index} => {},",
                generate_decode_statement(func, info.field_index, info.ownership_mode)
            )
        })
        .join("\n");

    let unwraps = (interest_fields.iter())
        .map(|info| {
            let field = &func.inputs[info.field_index];
            let mutability = if info.ownership_mode == OwnershipMode::RefMut {
                "mut "
            } else {
                ""
            };
            // "let {mutability}api_{name} = &{mutability}*api_{name}_guard.unwrap();\n",
            format!(
                "let {mutability}api_{name}_guard = api_{name}_guard.unwrap();\n",
                name = get_variable_name(field),
            )
        })
        .join("");

    format!(
        "{declarations}let decode_indices_ = flutter_rust_bridge::for_generated::lockable_compute_decode_order(vec![{var_orders}]);
        for i in decode_indices_ {{
            match i {{
                {match_arms}
                _ => unreachable!(),
            }}
        }}
        {unwraps}"
    )
}

fn generate_decode_statement(
    func: &MirFunc,
    field_index: usize,
    ownership_mode: OwnershipMode,
) -> String {
    let field = &func.inputs[field_index];
    let mode = ownership_mode.to_string().to_case(Case::Snake);
    format!(
        "api_{name}_guard = Some(api_{name}{maybe_illegal_static_ref}.lockable_decode_{syncness}_{mode}(){maybe_await})",
        name = get_variable_name(field),
        syncness = if func.rust_async { "async" } else { "sync" },
        maybe_await = if func.rust_async { ".await" } else { "" },
        maybe_illegal_static_ref = if field.needs_extend_lifetime { "_illegal_static_ref" } else { "" },
    )
}

fn get_variable_name(field: &MirFuncInput) -> String {
    field.inner.name.rust_style(true)
}

fn filter_interest_fields(func: &MirFunc) -> Vec<FieldInfo> {
    (func.inputs.iter())
        .enumerate()
        .filter_map(|(index, field)| {
            compute_interest_field_ownership_mode(&field.inner.ty).map(|ownership_mode| FieldInfo {
                field_index: index,
                ownership_mode,
            })
        })
        .collect_vec()
}

fn compute_interest_field_ownership_mode(ty: &MirType) -> Option<OwnershipMode> {
    match &ty {
        MirType::RustAutoOpaqueImplicit(ty)
            if ty.use_mutex && ty.ownership_mode != OwnershipMode::Owned =>
        {
            Some(ty.ownership_mode)
        }
        MirType::Delegate(MirTypeDelegate::ProxyEnum(ty)) => {
            compute_interest_field_ownership_mode(&ty.original)
        }
        // temporarily only support Ref
        MirType::Delegate(MirTypeDelegate::DynTrait(_)) => Some(OwnershipMode::Ref),
        MirType::Delegate(MirTypeDelegate::Lifetimeable(mir)) => {
            Some(if mir.api_type.ownership_mode == OwnershipMode::RefMut {
                OwnershipMode::RefMut
            } else {
                OwnershipMode::Ref
            })
        }
        _ => None,
    }
}

struct FieldInfo {
    field_index: usize,
    ownership_mode: OwnershipMode,
}

pub(crate) fn generate_inner_func_arg_ownership(field: &MirFuncInput) -> String {
    match &field.inner.ty {
        MirType::RustAutoOpaqueImplicit(_) => "".to_owned(),
        MirType::Delegate(MirTypeDelegate::DynTrait(_)) => "".to_owned(),
        _ => field
            .ownership_mode
            .map(|x| x.prefix())
            .unwrap_or_default()
            .to_owned(),
    }
}

pub(crate) fn generate_inner_func_arg(raw: &str, field: &MirFuncInput) -> String {
    if let Some(ownership_mode) = compute_interest_field_ownership_mode(&field.inner.ty) {
        let mutability = if ownership_mode == OwnershipMode::RefMut {
            "mut "
        } else {
            ""
        };
        return format!("&{mutability}*{raw}_guard");
    }

    if let MirType::RustAutoOpaqueImplicit(ty) = &field.inner.ty {
        if !ty.use_mutex && ty.ownership_mode == OwnershipMode::Ref {
            return format!("&*{raw}");
        }
    }

    raw.to_owned()
}
