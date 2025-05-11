mod db;
mod targets;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Attribute, BinOp, Expr, ExprAssign, ExprBinary, FnArg, Ident, ItemEnum, ItemFn, ItemMod,
    ItemStruct, Meta, ReturnType, Type, Visibility,
    parse::Parser,
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Const, EqEq, Static},
};

#[proc_macro_attribute]
pub fn wrapper_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let enum_def = parse_macro_input!(item as ItemEnum);

    let name = enum_def.ident.clone();

    let delayed_macros = enum_def.attrs.clone();

    let variants = enum_def.variants.iter().map(|variant| {
        let name = &variant.ident;
        if variant.fields.is_empty() {
            quote! {
                #name(#name),
            }
        } else {
            quote! {
                #variant,
            }
        }
    });

    let trait_into_name = Ident::new(format!("Into{}", name.to_string()).as_str(), name.span());
    let fn_into_name = Ident::new(
        format!("into_{}", name.to_string().to_lowercase()).as_str(),
        name.span(),
    );

    let parent = name.clone();
    let trait_impls = enum_def.variants.iter().map(|variant| {
        let name = &variant.ident;
        let inner_name = if variant.fields.is_empty() {
            name.to_token_stream()
        } else {
            variant
                .fields
                .iter()
                .next()
                .expect("Invalid wrapper type")
                .ty
                .to_token_stream()
        };
        quote! {
            impl #trait_into_name for #inner_name {
                fn #fn_into_name(self) -> #parent {
                    #parent::#name(self)
                }
            }
        }
    });

    let variant_unwrappers = enum_def.variants.iter().map(|variant| {
        let variant_name = variant.ident.clone();
        let inner_type = if variant.fields.is_empty() {
            variant.ident.to_token_stream()
        } else {
            variant
                .fields
                .iter()
                .next()
                .expect("Only use tuple wrappers!")
                .ty
                .to_token_stream()
        };
        let fn_name = Ident::new(
            format!("unwrap_{}", inner_type.to_string().to_lowercase()).as_str(),
            variant.span(),
        );
        let message = format!(
            "Attempting to unwrap {}::{}, but {}::{{other:?}} was contained within!",
            name.to_string(),
            variant_name.to_string(),
            name.to_string()
        );
        quote! {
            pub fn #fn_name(self) -> #inner_type {
                match self {
                    Self::#variant_name(value) => value,
                    other => panic!(#message),
                }
            }
        }
    });

    let expanded = quote! {
        #(#delayed_macros)*
        pub enum #name {
            #(#variants)*
        }
        impl #name {
            #(#variant_unwrappers)*
        }
        pub trait #trait_into_name {
            fn #fn_into_name(self) -> #name;
        }
        #(#trait_impls)*
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn bitmask_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let struct_def = parse_macro_input!(item as ItemStruct);
    let attr = Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated
        .parse(attr)
        .unwrap();

    let name = struct_def.ident.clone();

    let mut i = 0u8;
    let fields = attr
        .iter()
        .map(|ident| {
            let ident = ident.value();
            let set_ident = Ident::new(format!("set_{ident}").as_str(), Span::call_site());
            let is_ident = Ident::new(format!("is_{ident}").as_str(), Span::call_site());
            let mask = 0x1u8 << i;
            let not_mask = !mask;
            i += 1;
            let constant_ident = Ident::new(
                format!("BITMASK_{}", ident.to_uppercase()).as_str(),
                ident.span(),
            );
            quote! {
                pub const #constant_ident: u8 = #mask;
                pub fn #is_ident(&self) -> bool {
                    (self.mask & #mask) > 0
                }
                pub fn #set_ident(&mut self, value: bool) {
                    if value {
                        self.mask |= #mask;
                    } else {
                        self.mask &= #not_mask;
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        #struct_def
        impl #name {
            pub fn new() -> Self {
                Self::from_raw(0u8)
            }
            pub fn from_raw(mask: u8) -> Self {
                Self { mask }
            }
            #(#fields)*
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IntoSaturnus)]
pub fn derive_into_saturnus(items: TokenStream) -> TokenStream {
    let struct_def = parse_macro_input!(items as ItemStruct);
    let ident = struct_def.ident.clone();
    quote! {
        impl IntoSaturnus for #ident {
            fn into_saturnus(self) -> Any {
                self.into_any()
            }
        }
    }
    .into()
}

macro_rules! check {
    ( $left:pat => $right:expr, $( $a:pat => $b:expr ),+  ) => {
       check!($left => $right); $( check!($a => $b); )+
    };
    ( $left:pat => $right:expr ) => {
        let $left = $right else {
            return false;
        };
    };
}

fn is_st_proc(item_fn: &ItemFn) -> bool {
    let ItemFn { sig, .. } = item_fn;
    check! {
        ReturnType::Type(_, t) => &sig.output,
        Type::Path(tp) => &**t,
        Some(name) => tp.path.segments.first(),
        Visibility::Public(_) => &item_fn.vis
    }
    let mut input_iter = sig.inputs.iter();
    check! {
        Some(first) => input_iter.next(),
        FnArg::Typed(first) => first,
        Type::Path(first) => &*first.ty,
        Some(first) => first.path.get_ident()
    }
    // check! {
    //     Some(second) => input_iter.next(),
    //     FnArg::Typed(second) => second,
    //     Type::Path(second_type) => &*second.ty,
    //     Some(second) => second_type.path.get_ident(),
    //     Some(s_args) => second_type.path.segments.first()
    // }
    // let mut s_args = s_args.arguments.to_token_stream().into_iter();
    // check! {
    //     Some(_) => s_args.next(),
    //     Some(args) => s_args.next(),
    //     TokenTree::Ident(args) => args
    // }
    sig.asyncness.is_some() && name.ident.to_string() == "Result" && first.to_string() == "StVm"
    // && second.to_string() == "Vec"
    // && args.to_string() == "Any"
}

#[proc_macro_attribute]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mod_def = parse_macro_input!(item as ItemMod);
    let (_, mod_body) = mod_def.content.unwrap();
    let mod_ident = mod_def.ident;
    let mut targets = Vec::<Ident>::new();
    let mod_body = mod_body
        .into_iter()
        .map(|item| match item {
            syn::Item::Fn(item_fn) => {
                if is_st_proc(&item_fn) {
                    targets.push(item_fn.sig.ident.clone());
                }
                item_fn.to_token_stream()
            }
            _ => item.to_token_stream(),
        })
        .collect::<Vec<_>>();
    let lib_pub_body = targets
        .into_iter()
        .map(|item| {
            quote! {
                {
                    let key = vm.create_string(stringify!(#item));
                    let value = vm.create_fn(|vm, args| async move { #item(vm, args).await }).unwrap();
                    vm.set_table(&mut tbl, key, value);
                }
            }
        })
        .collect::<Vec<_>>();
    quote! {
        pub mod #mod_ident {
            #(#mod_body)*

            /// Autogenerated module loading procedure.
            pub fn load_mod(vm: &StVm) -> Table {
                let mut tbl = vm.create_table().unwrap();
                #(#lib_pub_body)*
                tbl
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn generate_bindings(attr: TokenStream, item: TokenStream) -> TokenStream {
    let exp = Punctuated::<ExprAssign, syn::Token![,]>::parse_terminated
        .parse(attr)
        .unwrap();
    for attr in exp {
        let Expr::Path(param) = &*attr.left else {
            panic!("Unknown expression found.");
        };
        let key = param
            .path
            .get_ident()
            .expect("Unexpected identifier")
            .to_string();
        match key.as_str() {
            "target" => return targets::compile_target(item, *attr.right),

            key => panic!("Unknown parameter '{key}'!"),
        }
    }
    quote! {
        compile_error!("No targets found!");
    }
    .into()
}
