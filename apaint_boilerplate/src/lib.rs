// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

extern crate proc_macro;

use heck::KebabCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

fn acronym(input: &str) -> String {
    let mut output = String::new();
    for char in input.chars() {
        if char.is_uppercase() {
            output.push(char);
        }
    }
    output
}

#[proc_macro_derive(Characteristic)]
pub fn characteristic_derive(input: TokenStream) -> TokenStream {
    let parsed_input: DeriveInput = parse_macro_input!(input);
    let enum_name = parsed_input.ident;
    let name = enum_name.to_string();
    let prompt = enum_name.to_string() + ":";
    let mut abbrev_tokens = vec![];
    let mut full_tokens = vec![];
    let mut from_tokens = vec![];
    let fmt_str = format!("\"{{}}\": Malformed '{}' value string", name);
    match parsed_input.data {
        Data::Enum(e) => {
            for v in e.variants {
                let v_name = v.ident.clone();
                let v_abbrev = acronym(&v.ident.to_string());
                let v_full = v.ident.to_string().to_kebab_case();
                let abbrev_token = quote! {
                    #enum_name::#v_name => #v_abbrev,
                };
                abbrev_tokens.push(abbrev_token);
                let full_token = quote! {
                    #enum_name::#v_name => #v_full,
                };
                full_tokens.push(full_token);
                let from_token = quote! {
                    #v_abbrev | #v_full => Ok(#enum_name::#v_name),
                };
                from_tokens.push(from_token);
            }
        }
        _ => panic!("'Characteristic' can only be derived for enums."),
    }
    let tokens = quote! {
        impl CharacteristicIfce for #enum_name {
            const NAME: &'static str = #name;
            const PROMPT: &'static str = #prompt;

            fn abbrev(&self) -> &'static str {
                match *self {
                    #(#abbrev_tokens)*
                }
            }

            fn full(&self) -> &'static str {
                match *self {
                    #(#full_tokens)*
                }
            }
        }

        impl std::str::FromStr for #enum_name {
            type Err = String;

            fn from_str(string: &str) -> Result<#enum_name, String> {
                match string {
                    #(#from_tokens)*
                    _ => Err(format!(#fmt_str, string)),
                }
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
