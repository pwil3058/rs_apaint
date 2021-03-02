// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

extern crate proc_macro;

use heck::KebabCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident};

fn acronym(input: &str) -> String {
    let mut output = String::new();
    for char in input.chars() {
        if char.is_uppercase() {
            output.push(char);
        }
    }
    output
}

fn abbreviate(input: &str, n: usize) -> String {
    let mut output = String::new();
    for (i, char) in input.chars().enumerate() {
        if i < n {
            output.push(char);
        } else {
            break;
        }
    }
    output.push('.');
    output
}

#[proc_macro_derive(Characteristic, attributes(default))]
pub fn characteristic_derive(input: TokenStream) -> TokenStream {
    let parsed_input: DeriveInput = parse_macro_input!(input);
    let enum_name = parsed_input.ident;
    let name = enum_name.to_string();
    let prompt = enum_name.to_string() + ":";
    let list_header_name = abbreviate(&enum_name.to_string(), 2);
    let mut abbrev_tokens = vec![];
    let mut full_tokens = vec![];
    let mut from_tokens = vec![];
    let mut from_f64_tokens = vec![];
    let mut to_f64_tokens = vec![];
    let mut value_tokens = vec![];
    let mut first: Option<Ident> = None;
    let mut default: Option<Ident> = None;
    let fmt_str = format!("\"{{}}\": Malformed '{}' value string", name);
    match parsed_input.data {
        Data::Enum(e) => {
            let mut count: u64 = 1;
            for v in e.variants {
                let v_name = v.ident.clone();
                if first.is_none() {
                    first = Some(v.ident.clone());
                }
                for attr in v.attrs.iter() {
                    if attr.path.is_ident("default") {
                        default = Some(v.ident.clone());
                    }
                }
                let v_abbrev = acronym(&v.ident.to_string());
                let v_full = v.ident.to_string().to_kebab_case();
                let token = quote! {
                    #v_full,
                };
                value_tokens.push(token);
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
                let from_f64_token = quote! {
                    #count => #enum_name::#v_name,
                };
                from_f64_tokens.push(from_f64_token);
                let to_f64_token = quote! {
                    #enum_name::#v_name => #count as f64,
                };
                to_f64_tokens.push(to_f64_token);
                count += 1;
            }
        }
        _ => panic!("'Characteristic' can only be derived for enums."),
    }
    let default_value = if let Some(default) = default {
        default
    } else {
        first.unwrap()
    };
    let tokens = quote! {
        impl CharacteristicIfce for #enum_name {
            const NAME: &'static str = #name;
            const PROMPT: &'static str = #prompt;
            const LIST_HEADER_NAME: &'static str = #list_header_name;

            fn str_values() -> Vec<&'static str> {
                vec![#(#value_tokens)*]
            }

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

        impl std::convert::From<f64> for #enum_name {
            fn from(float: f64) -> #enum_name {
                match float.round() as u64 {
                    #(#from_f64_tokens)*
                    _ => panic!("u64: {} out of range for '{}'", float, #name),
                }
            }
        }

        impl std::convert::From<#enum_name> for f64 {
            fn from(arg: #enum_name) -> f64 {
                match arg {
                    #(#to_f64_tokens)*
                }
            }
        }

        impl std::default::Default for #enum_name {
            fn default() -> Self { #enum_name::#default_value }
        }
    };

    proc_macro::TokenStream::from(tokens)
}

#[proc_macro_derive(BasicPaint, attributes(colour))]
pub fn basic_paint_interface_derive(input: TokenStream) -> TokenStream {
    let parsed_input: DeriveInput = parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    let (impl_generics, ty_generics, where_clause) = parsed_input.generics.split_for_impl();
    let tokens = quote! {
        impl #impl_generics crate::BasicPaintIfce for #struct_name #ty_generics #where_clause {
            fn id(&self) -> &str {
                &self.id
            }

            fn name(&self) -> Option<&str> {
                if self.name.len() == 0 {
                    None
                } else {
                    Some(&self.name)
                }
            }

            fn notes(&self) -> Option<&str> {
                if self.notes.len() == 0 {
                    None
                } else {
                    Some(&self.notes)
                }
            }

            fn finish(&self) -> Finish {
                self.finish
            }

            fn transparency(&self) -> Transparency {
                self.transparency
            }

            fn fluorescence(&self) -> Fluorescence {
                self.fluorescence
            }

            fn permanence(&self) -> Permanence {
                self.permanence
            }

            fn metallicness(&self) -> Metallicness {
                self.metallicness
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
