// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

extern crate proc_macro;

use heck::KebabCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
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
    let mut value_tokens = vec![];
    let mut first: Option<Ident> = None;
    let mut default: Option<Ident> = None;
    let fmt_str = format!("\"{{}}\": Malformed '{}' value string", name);
    match parsed_input.data {
        Data::Enum(e) => {
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

        impl std::default::Default for #enum_name {
            fn default() -> Self { #enum_name::#default_value }
        }
    };

    proc_macro::TokenStream::from(tokens)
}

#[proc_macro_derive(Colour, attributes(colour, component))]
pub fn colour_interface_derive(input: TokenStream) -> TokenStream {
    let parsed_input: DeriveInput = parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    let (impl_generics, ty_generics, where_clause) = parsed_input.generics.split_for_impl();
    let mut first: Option<Ident> = None;
    let mut colour: Option<Ident> = None;
    match parsed_input.data {
        Data::Struct(st) => {
            if let syn::Fields::Named(fields) = st.fields {
                for field in fields.named.iter() {
                    if first.is_none() {
                        first = Some(field.ident.clone().unwrap());
                    }
                    for attr in field.attrs.iter() {
                        if attr.path.is_ident("colour") {
                            colour = Some(field.ident.clone().unwrap());
                        }
                    }
                }
            }
        }
        _ => panic!("'Colour' can only be derived for structs."),
    }
    let colour = if let Some(colour) = colour {
        colour
    } else {
        first.unwrap()
    };
    let mut component = { Ident::new("F", struct_name.span()) };
    for attr in parsed_input.attrs.iter() {
        if attr.path.is_ident("component") {
            if let Ok(meta) = attr.parse_meta() {
                match meta {
                    syn::Meta::NameValue(nv) => {
                        if let syn::Lit::Str(lit_str) = &nv.lit {
                            component = Ident::new(&lit_str.value(), nv.span());
                        }
                    }
                    _ => panic!("expected 'component = name'"),
                }
            }
        }
    }
    let tokens = quote! {
        impl #impl_generics colour_math::ColourInterface<#component> for #struct_name #ty_generics #where_clause {
            fn rgb(&self) -> colour_math::RGB<#component> {
                self.#colour.rgb()
            }

            fn rgba(&self, alpha: #component) -> [#component; 4] {
                self.#colour.rgba(alpha)
            }

            fn hue(&self) -> Option<colour_math::Hue<#component>> {
                self.#colour.hue()
            }

            fn hue_angle(&self) -> Option<colour_math::Degrees<#component>> {
                self.#colour.hue_angle()
            }

            fn is_grey(&self) -> bool {
                self.#colour.is_grey()
            }

            fn chroma(&self) -> #component {
                self.#colour.chroma()
            }

            fn greyness(&self) -> #component {
                self.#colour.greyness()
            }

            fn value(&self) -> #component {
                self.#colour.value()
            }

            fn warmth(&self) -> #component {
                self.#colour.warmth()
            }

            fn best_foreground_rgb(&self) -> colour_math::RGB<#component> {
                self.#colour.best_foreground_rgb()
            }

            fn monochrome_rgb(&self) -> colour_math::RGB<#component> {
                self.#colour.monochrome_rgb()
            }

            fn max_chroma_rgb(&self) -> colour_math::RGB<#component> {
                self.#colour.max_chroma_rgb()
            }

            fn warmth_rgb(&self) -> colour_math::RGB<#component> {
                self.#colour.warmth_rgb()
            }

            fn scalar_attribute(&self, attr: ScalarAttribute) -> #component {
                self.#colour.scalar_attribute(attr)
            }

            fn scalar_attribute_rgb(&self, attr: ScalarAttribute) -> colour_math::RGB<#component> {
                self.#colour.scalar_attribute_rgb(attr)
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
