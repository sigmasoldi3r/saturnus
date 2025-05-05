use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    AttributeArgs, Ident, ItemEnum, ItemMod, ItemStruct, NestedMeta, parse_macro_input,
    spanned::Spanned,
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
    let attr = parse_macro_input!(attr as AttributeArgs);

    let name = struct_def.ident.clone();

    let mut i = 0u8;
    let fields = attr.iter().map(|ident| {
        let NestedMeta::Meta(meta) = ident else {
            return quote!(compile_error!("Should use only identifiers as parameters!"));
        };
        let ident = meta.path().get_ident().unwrap();
        let set_ident = Ident::new(format!("set_{}", ident.to_string()).as_str(), ident.span());
        let is_ident = Ident::new(format!("is_{}", ident.to_string()).as_str(), ident.span());
        let mask = 0x1u8 << i;
        let not_mask = !mask;
        i += 1;
        let constant_ident = Ident::new(
            format!("BITMASK_{}", ident.to_string().to_uppercase()).as_str(),
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
    });

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
                if item_fn.sig.asyncness.is_some() {
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
                    let key = vm.lock().create_string(stringify!(#item));
                    let value = vm.lock().create_fn(#item).unwrap();
                    vm.lock().set_table(&mut tbl, key, value);
                }
            }
        })
        .collect::<Vec<_>>();
    quote! {
        pub mod #mod_ident {
            #(#mod_body)*

            /// Autogenerated module loading procedure.
            pub fn load_mod(vm: St<StVm>) -> Table {
                let mut tbl = vm.lock().create_table().unwrap();
                #(#lib_pub_body)*
                tbl
            }
        }
    }
    .into()
}
