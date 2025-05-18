use quote::ToTokens;
use syn::{Attribute, Expr, Meta};

pub fn pascal_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn get_default_value(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
    for attr in attrs {
        if attr.path().is_ident("builder") {
            let meta: Meta = attr.parse_args().ok()?;
            if let Meta::NameValue(mnv) = meta {
                if mnv.path.is_ident("default") {
                    if let Expr::Lit(expr_lit) = mnv.value {
                        return Some(expr_lit.to_token_stream());
                    } else {
                        return Some(mnv.value.to_token_stream());
                    }
                }
            }
        }
    }
    None
}
