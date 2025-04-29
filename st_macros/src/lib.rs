use proc_macro::TokenStream;
use quote::quote;
use syn::{AttributeArgs, Ident, ItemEnum, ItemStruct, NestedMeta, parse_macro_input};

#[proc_macro_attribute]
pub fn wrapper_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let enum_def = parse_macro_input!(item as ItemEnum);

    let name = enum_def.ident.clone();

    let delayed_macros = enum_def.attrs.clone();

    let variants = enum_def.variants.iter().map(|variant| {
        let name = &variant.ident;
        quote! {
            #name(#name),
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
        quote! {
            impl #trait_into_name for #name {
                fn #fn_into_name(self) -> #parent {
                    #parent::#name(self)
                }
            }
        }
    });

    let variant_unwrappers = enum_def.variants.iter().map(|variant| {
        let inner_type = variant.ident.clone();
        let fn_name = Ident::new(
            format!("unwrap_{}", inner_type.to_string().to_lowercase()).as_str(),
            inner_type.span(),
        );
        let message = format!(
            "Attempting to unwrap {}::{}, but other type was contained within!",
            name.to_string(),
            inner_type.to_string()
        );
        quote! {
            pub fn #fn_name(self) -> #inner_type {
                match self {
                    Self::#inner_type(value) => value,
                    _ => panic!(#message),
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
