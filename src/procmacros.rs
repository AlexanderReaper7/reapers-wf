use core::panic;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data};

#[proc_macro_derive(Display)]
pub fn proc_macro_derive_display(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Enum(ref innerdata) => {
            let var = innerdata.variants.iter().map(|v| {
                let name = &v.ident;
                let name_str = name.to_string();
                quote!{
                    Self::#name => write!(f, #name_str),
                }
            });
            let enum_name_str = &input.ident.to_string().split(" ").into_iter().last().unwrap().to_owned();
            let enum_name = proc_macro2::Ident::new(enum_name_str, Span::call_site()) ;
            let expanded = quote!{
                impl std::fmt::Display for #enum_name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#var)*
                        }
                    }
                }
            };
            TokenStream::from(expanded)
        }
        _ => panic!("Display can only be derived for enums"),
    }
}
#[proc_macro_derive(FromStr)]
pub fn proc_macro_derive_fromstr(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Enum(ref innerdata) => {
            let var = innerdata.variants.iter().map(|v| {
                let name = &v.ident;
                let name_str = name.to_string();
                quote! {
                    #name_str => Ok(Self::#name),
                }
            });
            let enum_name_str = &input.ident.to_string().split(" ").into_iter().last().unwrap().to_owned();
            let enum_name = proc_macro2::Ident::new(enum_name_str, Span::call_site()) ;
            let expanded = quote! {
                impl std::str::FromStr for #enum_name {
                    type Err = String;

                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        match s {
                            #(#var)*
                            _ => Err(format!("{} is not a valid {}", s, stringify!(#enum_name))),
                        }
                    }
                }
            };

            TokenStream::from(expanded)
        }
        _ => panic!("FromStr can only be derived for enums"),
    }
}