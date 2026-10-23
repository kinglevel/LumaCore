//! `#[derive(Args)]`: a verb's typed arguments are its signature (decisions §33 decision 4).
//!
//! ```ignore
//! #[derive(Args)]
//! #[lum(verb = "Set")]
//! struct SetProperty {
//!     #[lum(target)] target: Target,          // Option<Target> = may be left out
//!     #[lum(property)] property: String,      // "Name" — quoted (decision 7)
//!     #[lum(value)] value: Value,             // binds from the right (decision 17)
//!     #[lum(helper = "Fade")] fade: Option<Value>,
//!     #[lum(at = "destination")] to: Value,   // `At` as where to (decision 8); `#[lum(at)]` = a value
//!     #[lum(option = "Overwrite")] overwrite: bool,       // a switch
//!     #[lum(option = "Universe")] universe: Option<Value>, // takes a value
//!     #[lum(phase)] phase: Phase,             // a hold verb's On / Off
//!     #[lum(condition)] filter: Option<Condition>,
//! }
//! ```
//!
//! The derive writes `impl lumacore_syntax::Args` (the `SIGNATURE` const and `from_bound`) and a
//! `const` that runs `Signature::check` when the crate compiles: a helper or option declared
//! twice, or two optional plain slots that can't be told apart, is a compile error:
//!
//! ```compile_fail
//! use lumacore_syntax_derive::Args;
//! use lumacore_syntax::ast::Value;
//! #[derive(Args)]
//! #[lum(verb = "Store")]
//! struct Twice {
//!     #[lum(value)] a: Option<Value>,
//!     #[lum(value)] b: Option<Value>,
//! }
//! ```
//!
//! ```compile_fail
//! use lumacore_syntax_derive::Args;
//! #[derive(Args)]
//! #[lum(verb = "Store")]
//! struct Twice {
//!     #[lum(option = "Merge")] a: bool,
//!     #[lum(option = "merge")] b: bool,
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Type, parse_macro_input, spanned::Spanned};

#[proc_macro_derive(Args, attributes(lum))]
pub fn derive_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

enum Part {
    Target,
    Slot(&'static str),
    Helper { word: String, destination: bool },
    Option(String),
    Phase,
    Condition,
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new(input.generics.span(), "Args structs take no generics"));
    }
    let (mut verb, mut object) = (None::<LitStr>, None::<LitStr>);
    for attr in input.attrs.iter().filter(|a| a.path().is_ident("lum")) {
        attr.parse_nested_meta(|m| {
            if m.path.is_ident("verb") {
                verb = Some(m.value()?.parse()?);
            } else if m.path.is_ident("object") {
                object = Some(m.value()?.parse()?);
            } else {
                return Err(m.error("expected `verb = \"…\"` or `object = \"…\"`"));
            }
            Ok(())
        })?;
    }
    let verb = verb.ok_or_else(|| syn::Error::new(Span::call_site(), "#[lum(verb = \"…\")] names the verb"))?;
    let object = match &object {
        Some(o) => quote!(::core::option::Option::Some(#o)),
        None => quote!(::core::option::Option::None),
    };

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(Span::call_site(), "Args is derived for a struct"));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new(data.fields.span(), "Args fields are named"));
    };

    let lib = quote!(::lumacore_syntax::bind);
    let mut target = quote!(#lib::Need::Never);
    let mut phase = false;
    let mut condition = false;
    let (mut slots, mut helpers, mut options, mut inits) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut helper_names, mut option_names) = (Vec::<String>::new(), Vec::<String>::new());
    let mut seen_target = false;

    for field in &fields.named {
        let ident = field.ident.as_ref().expect("named");
        let ty = &field.ty;
        let field_name = ident.to_string();
        let mut part = None;
        for attr in field.attrs.iter().filter(|a| a.path().is_ident("lum")) {
            attr.parse_nested_meta(|m| {
                let p = if m.path.is_ident("target") {
                    Part::Target
                } else if m.path.is_ident("property") {
                    Part::Slot("Property")
                } else if m.path.is_ident("name") {
                    Part::Slot("Name")
                } else if m.path.is_ident("value") {
                    Part::Slot("Value")
                } else if m.path.is_ident("phase") {
                    Part::Phase
                } else if m.path.is_ident("condition") {
                    Part::Condition
                } else if m.path.is_ident("at") {
                    let destination = if m.input.peek(syn::Token![=]) {
                        let role: LitStr = m.value()?.parse()?;
                        match role.value().as_str() {
                            "destination" => true,
                            "value" => false,
                            _ => return Err(syn::Error::new(role.span(), "`at` is \"value\" or \"destination\"")),
                        }
                    } else {
                        false
                    };
                    Part::Helper { word: "At".into(), destination }
                } else if m.path.is_ident("helper") {
                    let word: LitStr = m.value()?.parse()?;
                    Part::Helper { word: word.value(), destination: false }
                } else if m.path.is_ident("option") {
                    let o: LitStr = m.value()?.parse()?;
                    Part::Option(o.value())
                } else {
                    return Err(m.error("expected target, property, name, value, at, helper, option, phase or condition"));
                };
                if part.replace(p).is_some() {
                    return Err(m.error("one #[lum(…)] part per field"));
                }
                Ok(())
            })?;
        }
        let Some(part) = part else {
            return Err(syn::Error::new(field.span(), "every field says which part it is: #[lum(…)]"));
        };
        match part {
            Part::Target => {
                if seen_target {
                    return Err(syn::Error::new(field.span(), "one target per statement (decision 20)"));
                }
                seen_target = true;
                target = quote!(if <#ty as #lib::FromTarget>::REQUIRED { #lib::Need::Required } else { #lib::Need::Optional });
                inits.push(quote!(#ident: <#ty as #lib::FromTarget>::from_target(bound.target.take(), end)?));
            }
            Part::Slot(kind) => {
                let i = slots.len();
                let kind = syn::Ident::new(kind, Span::call_site());
                slots.push(quote!(#lib::SlotDef { kind: #lib::SlotKind::#kind, required: <#ty as #lib::FromValue>::REQUIRED, field: #field_name }));
                inits.push(quote!(#ident: <#ty as #lib::FromValue>::from_value(bound.slots[#i].take(), end, vocab)?));
            }
            Part::Helper { word, destination } => {
                if helper_names.iter().any(|h| h.eq_ignore_ascii_case(&word)) {
                    return Err(syn::Error::new(field.span(), format!("the helper {word} is declared twice")));
                }
                helper_names.push(word.clone());
                let i = helpers.len();
                let role = if destination { quote!(#lib::Role::Destination) } else { quote!(#lib::Role::Value) };
                helpers.push(quote!(#lib::HelperDef { word: #word, role: #role, required: <#ty as #lib::FromValue>::REQUIRED, field: #field_name }));
                inits.push(quote!(#ident: <#ty as #lib::FromValue>::from_value(bound.helpers[#i].take(), end, vocab)?));
            }
            Part::Option(o) => {
                if option_names.iter().any(|n| n.eq_ignore_ascii_case(&o)) {
                    return Err(syn::Error::new(field.span(), format!("the option /{o} is declared twice")));
                }
                option_names.push(o.clone());
                let i = options.len();
                let switch = is_bool(ty);
                options.push(quote!(#lib::OptionSig { name: #o, takes_value: !#switch }));
                inits.push(if switch {
                    quote!(#ident: bound.options[#i].is_some())
                } else {
                    quote!(#ident: <#ty as #lib::FromValue>::from_value(bound.options[#i].take().flatten(), end, vocab)?)
                });
            }
            Part::Phase => {
                if phase {
                    return Err(syn::Error::new(field.span(), "one phase per statement"));
                }
                phase = true;
                inits.push(quote!(#ident: <#ty as #lib::FromPhase>::from_phase(bound.phase.take(), end)?));
            }
            Part::Condition => {
                if condition {
                    return Err(syn::Error::new(field.span(), "one If per statement"));
                }
                condition = true;
                inits.push(quote!(#ident: bound.condition.take()));
            }
        }
    }

    Ok(quote! {
        impl #lib::Args for #name {
            const SIGNATURE: #lib::Signature = #lib::Signature {
                verb: #verb,
                object: #object,
                target: #target,
                phase: #phase,
                slots: &[#(#slots),*],
                helpers: &[#(#helpers),*],
                options: &[#(#options),*],
                condition: #condition,
            };

            #[allow(unused_mut)]
            fn from_bound(
                mut bound: #lib::Bound,
                vocab: &::lumacore_syntax::Vocabulary,
            ) -> ::core::result::Result<Self, ::lumacore_syntax::Diag> {
                let end = bound.end;
                let _ = (end, vocab);
                ::core::result::Result::Ok(Self { #(#inits),* })
            }
        }

        const _: () = <#name as #lib::Args>::SIGNATURE.check();
    })
}

fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Path(p) if p.qself.is_none() && p.path.is_ident("bool"))
}
