use proc_macro::TokenStream;
use proc_macro2::{Group, Span, extra::DelimSpan};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Block, Data, DataStruct, DeriveInput, Expr, Generics, Ident, ImplItem, ItemFn, Lit,
    PatType, PathArguments, Signature, Type, parse_macro_input,
    punctuated::Punctuated,
    token::{Async, Colon, Const, Mut, Paren, Pub, RArrow, Ref, Unsafe},
};
use toml::toml;

use crate::db::{Db, DynErr, ToDynErr};

fn convert_to_db(item: ImplItem) -> toml::Value {
    match item {
        ImplItem::Const(impl_item_const) => {
            toml! {
                variant = "const"
                name = (impl_item_const.ident.to_string())
                type_def = (impl_item_const.ty.to_token_stream().to_string())
            }
        }
        ImplItem::Fn(impl_item_fn) => {
            let mut inputs: Vec<toml::Value> = vec![];
            let mut rec_info: Option<toml::Value> = None;
            for input in impl_item_fn.sig.inputs.iter() {
                match input {
                    syn::FnArg::Receiver(receiver) => {
                        rec_info = Some(
                            toml! {
                                is_mutable = (receiver.mutability.is_some())
                                is_ref = (receiver.reference.is_some())
                            }
                            .into(),
                        );
                    }
                    syn::FnArg::Typed(pat_type) => inputs.push(
                        toml! {
                            name = (pat_type.pat.clone().into_token_stream().to_string())
                            type_def = (pat_type.ty.clone().to_token_stream().to_string())
                        }
                        .into(),
                    ),
                }
            }
            let rec_info = rec_info.unwrap_or(
                toml! {
                    [no_value]
                }
                .into(),
            );
            let output = match impl_item_fn.sig.output {
                syn::ReturnType::Default => "".to_string(),
                syn::ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
            };
            let attr = impl_item_fn
                .attrs
                .iter()
                .map(|attr| attr.to_token_stream().to_string())
                .collect::<Vec<_>>();
            toml! {
                variant = "fn"
                attr = (attr)
                receiver = (rec_info)
                name = (impl_item_fn.sig.ident.to_string())
                is_async = (impl_item_fn.sig.asyncness.is_some())
                is_const = (impl_item_fn.sig.constness.is_some())
                is_unsafe = (impl_item_fn.sig.unsafety.is_some())
                vis = (impl_item_fn.vis.clone().to_token_stream().to_string())
                output = (output)
                inputs = (inputs)
            }
        }
        other => {
            toml! {
                variant = "unknown"
                raw = (format!("{other:?}"))
            }
        }
    }
    .into()
}

fn fetch_info_lua() -> Result<(), DynErr> {
    let mut db = Db::load()?;
    if !db.is_empty() {
        return Ok(());
    }
    let res = reqwest::blocking::get(
        "https://raw.githubusercontent.com/mlua-rs/mlua/refs/heads/v0.10/src/state.rs",
    )
    .to_dyn()?;
    let raw = res.text().to_dyn()?;
    let ast: syn::File = syn::parse_str(raw.as_str()).to_dyn()?;
    for item in ast.items {
        let syn::Item::Impl(impl_block) = item else {
            continue;
        };
        let syn::Type::Path(path_name) = &*impl_block.self_ty else {
            continue;
        };
        let Some(name) = path_name.path.get_ident() else {
            continue;
        };
        if name.to_string() != "Lua" {
            continue;
        }
        // Store the lua impl block!
        let ns = db.get_table_space("lua_bindgen");
        let lua_impl_info = ns.get_table("lua_impl_block");
        for item in impl_block.items {
            lua_impl_info.push(convert_to_db(item));
        }
        db.save()?;
    }
    Ok(())
}

fn is_trivial_type(type_def: &Type) -> bool {
    match type_def {
        Type::Tuple(dfs) => dfs.elems.is_empty() || dfs.elems.iter().all(is_trivial_type),
        Type::Never(_) => true,
        Type::Path(pth) => {
            let seg = &pth.path.segments;
            let Some(name) = seg.first() else {
                return false;
            };
            match name.ident.to_string().as_str() {
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
                | "u128" | "usize" | "f32" | "f64" | "char" | "str" | "bool" | "String"
                | "c_int" => {
                    return true;
                }
                _ => {
                    if name.ident.to_string() != "Result" {
                        return false;
                    }
                    let PathArguments::AngleBracketed(inner) = &name.arguments else {
                        return false;
                    };
                    inner.args.iter().all(|ty| match ty {
                        syn::GenericArgument::Type(ty) => is_trivial_type(ty),
                        _ => false,
                    })
                }
            }
        }
        _ => false,
    }
}

fn make_const(val: bool) -> Option<Const> {
    if val {
        Some(Const {
            span: Span::call_site(),
        })
    } else {
        None
    }
}
fn make_unsafe(val: bool) -> Option<Unsafe> {
    if val {
        Some(Unsafe {
            span: Span::call_site(),
        })
    } else {
        None
    }
}
fn make_async(val: bool) -> Option<Async> {
    if val {
        Some(Async {
            span: Span::call_site(),
        })
    } else {
        None
    }
}

trait TracingErr<T> {
    fn trace(self) -> Option<T>;
}
impl<T> TracingErr<T> for Result<T, DynErr> {
    fn trace(self) -> Option<T> {
        match self {
            Ok(ok) => Some(ok),
            Err(err) => {
                eprintln!("{err:?}");
                None
            }
        }
    }
}

trait ParsingUtils {
    fn parse_as<T: syn::parse::Parse>(&self) -> Result<T, DynErr>;
}
impl ParsingUtils for String {
    fn parse_as<T: syn::parse::Parse>(&self) -> Result<T, DynErr> {
        syn::parse(self.parse::<TokenStream>().to_dyn()?).to_dyn()
    }
}

trait OrUnit {
    fn or_unit(self) -> Self;
}
impl OrUnit for String {
    fn or_unit(self) -> Self {
        if self.is_empty() {
            return "()".to_string();
        }
        self
    }
}

fn translate_comment(from: String) -> String {
    from.replace("Lua", "Saturnus")
}

fn extract_trivially_bindable(parent_name: String, db: &mut Db) -> Vec<proc_macro2::TokenStream> {
    let ns = db.get_table_space("lua_bindgen");
    let lua_impl_info = ns.get_table("lua_impl_block");
    lua_impl_info
        .iter()
        .filter_map(|def| {
            let variant = def.get("variant")?.as_str()?.to_string();
            if variant != "fn" {
                return None;
            }
            if def.get("vis")?.as_str()?.to_string() != "pub" {
                return None;
            }
            let ident = def
                .get("name")?
                .as_str()?
                .to_string()
                .parse_as::<Ident>()
                .trace()?;
            let output = def
                .get("output")?
                .as_str()?
                .to_string()
                .or_unit()
                .parse_as::<Type>()
                .unwrap();
            if !is_trivial_type(&output) {
                return None;
            }
            let is_const = def.get("is_const")?.as_bool()?;
            let is_async = def.get("is_async")?.as_bool()?;
            let is_unsafe = def.get("is_unsafe")?.as_bool()?;
            struct Param {
                name: String,
                type_def: Type,
            }
            let inputs = def
                .get("inputs")?
                .as_array()?
                .iter()
                .filter_map(|item| {
                    let name = item.get("name")?.as_str()?.to_string();
                    let type_def = item.get("type_def")?.as_str()?.to_string();
                    Some(Param {
                        name,
                        type_def: type_def.parse_as::<Type>().trace()?,
                    })
                })
                .collect::<Vec<_>>();
            if inputs.iter().any(|def| !is_trivial_type(&def.type_def)) {
                return None;
            }
            let param = inputs
                .iter()
                .map(|param| {
                    let name = param.name.parse_as::<Ident>().unwrap();
                    let ty = &param.type_def;
                    quote! {
                        #name: #ty
                    }
                })
                .collect::<Vec<_>>();
            let call_args = inputs
                .iter()
                .map(|param| param.name.parse_as::<Ident>().unwrap())
                .collect::<Vec<_>>();
            let constness = make_const(is_const);
            let asyncness = make_async(is_async);
            let unsafety = make_unsafe(is_unsafe);
            let rec_info = {
                let rec = def.get("receiver")?;
                let is_mut = rec.get("is_mutable")?.as_bool()?;
                let is_ref = rec.get("is_ref")?.as_bool()?;
                let mut_token = if is_mut {
                    quote! { mut }
                } else {
                    quote! {}
                };
                let ref_token = if is_ref {
                    quote! { & }
                } else {
                    quote! {}
                };
                quote! {
                    #ref_token #mut_token self
                }
            };
            let args = vec![rec_info.clone()]
                .into_iter()
                .chain(param.clone().into_iter())
                .collect::<Vec<_>>();
            let raw_attr = def
                .get("attr")
                .map(|attr| {
                    attr.as_array()
                        .unwrap()
                        .into_iter()
                        .map(|val| val.as_str().unwrap().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            // If the method is gated behid something unreachable, skip it.
            if raw_attr
                .iter()
                .any(|attr| attr.contains("#[cfg") && !attr.contains("feature = lua53"))
            {
                return None;
            }
            // Else collect the attributes and fix the docs.
            let attr = raw_attr
                .into_iter()
                .map(|attr| {
                    let attr = if attr.contains("#[doc") {
                        attr.replace("Lua", "Saturnus")
                            .replace("mlua", "saturnus")
                            .replace("lua", "sat")
                    } else {
                        attr
                    };
                    let stmt = format!("{attr}\nfn _foo() {{}}")
                        .parse_as::<syn::Stmt>()
                        .unwrap();
                    match stmt {
                        syn::Stmt::Item(item) => match item {
                            syn::Item::Fn(ItemFn { mut attrs, .. }) => attrs.remove(0),
                            _ => unimplemented!(),
                        },
                        _ => unimplemented!(),
                    }
                })
                .collect::<Vec<_>>();
            let docs = attr
                .iter()
                .filter(|attr| attr.meta.path().get_ident().unwrap().to_string() == "doc")
                .collect::<Vec<_>>();
            loop {
                let Type::Path(output) = &output else {
                    break;
                };
                let Some(first) = output.path.segments.first() else {
                    break;
                };
                if first.ident.to_string() != "Result" {
                    break;
                };
                let message = format!(
                    "Internal VM runtime error while calling {}::{}(..)!",
                    parent_name,
                    ident.to_string()
                );
                return Some(quote! {
                    #( #docs )*
                    pub #unsafety #constness #asyncness fn #ident(#( #args ),*) -> #output {
                        match self.runtime.#ident(#( #call_args ),*) {
                            Ok(value) => Ok(value),
                            Err(err) => Err(RuntimeError {
                                message: #message.into(),
                                caused_by: Some(Box::new(err)),
                            }),
                        }
                    }
                });
            }
            Some(quote! {
                #( #docs )*
                pub #unsafety #constness #asyncness fn #ident(#( #args ),*) -> #output {
                    self.runtime.#ident(#( #call_args ),*)
                }
            })
        })
        .collect()
}

fn compile_lua(item: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = parse_macro_input!(item as DeriveInput);
    let Data::Struct(DataStruct {
        struct_token,
        fields,
        ..
    }) = data
    else {
        return quote! {
            compile_error!("Can't generate bindings for non-struct data containers.");
        }
        .into();
    };
    let fields = fields.iter().map(Clone::clone).collect::<Vec<_>>();
    let field_names = fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect::<Vec<_>>();
    // Now, query the data from the db
    fetch_info_lua().expect("Something went wrong! Cannot acquire bindings data!");
    // First generate wrappers for trivial bindings
    let mut db = Db::load().expect("Cannot open database!");
    let trivially_bindable = extract_trivially_bindable(ident.to_string(), &mut db);
    quote! {
        #( #attrs )*
        ///
        /// *Note*: Generated bindings. See `#[generate_bindings(..)` macro attribute.
        #vis #struct_token #ident {
            /// Generated runtime wrapper.
            runtime: mlua::Lua,

            #( #fields )*
        }

        impl #ident {
            pub fn new() -> Self {
                Self {
                    runtime: mlua::Lua::new(),
                    #( #field_names: Default::default(), )*
                }
            }
            #( #trivially_bindable )*
        }

        #vis type Result<T> = std::result::Result<T, RuntimeError>;

        #[derive(Debug)]
        #vis struct RuntimeError {
            pub caused_by: Option<Box<dyn std::error::Error>>,
            pub message: String,
        }
        impl std::fmt::Display for RuntimeError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let maybe_cause = if let Some(cause) = &self.caused_by {
                    format!(". Caused by {}", cause)
                } else {
                    "".to_string()
                };
                write!(f, "Runtime error: {}{}", self.message, maybe_cause)
            }
        }
        impl std::error::Error for RuntimeError {}
    }
    .into()
}

pub fn compile_target(item: TokenStream, target: Expr) -> TokenStream {
    let target = {
        let Expr::Lit(lit) = target else {
            panic!("Unknown target!");
        };
        let Lit::Str(val) = lit.lit else {
            panic!("Unknown target!");
        };
        val.value()
    };
    match target.as_str() {
        "Lua" => return compile_lua(item),
        _ => (),
    }
    quote! {
        compile_error!("Unknown target!");
    }
    .into()
}
