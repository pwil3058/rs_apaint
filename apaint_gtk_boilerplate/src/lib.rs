// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Wrapper)]
pub fn wrapper_derive(input: TokenStream) -> TokenStream {
    let parsed_input: syn::DeriveInput = syn::parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    let _name = struct_name.to_string();
    let (impl_generics, ty_generics, where_clause) = parsed_input.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics PackableWidgetObject for #struct_name #ty_generics #where_clause {
            type PWT = whatever;

            fn pwo(&self) -> Self::PWT {
                self.first.clone()
            }
        }
    };
    proc_macro::TokenStream::from(tokens)
}
