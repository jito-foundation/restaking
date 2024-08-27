use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Lit, parse::Parse, punctuated::Punctuated, Token};

struct Args {
    discriminator: u64,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content: Punctuated<Lit, Token![,]> = input.parse_terminated(Lit::parse, Token![,])?;
        if let Some(Lit::Int(lit)) = content.first() {
            Ok(Args {
                discriminator: lit.base10_parse()?,
            })
        } else {
            Err(syn::Error::new(input.span(), "Expected an integer literal"))
        }
    }
}

#[proc_macro_attribute]
pub fn event(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let args = parse_macro_input!(args as Args);

    let discriminator = args.discriminator;
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        #input

        impl #impl_generics Event for #name #type_generics #where_clause {
            const DISCRIMINATOR: EventDiscriminator = EventDiscriminator::new(#discriminator);
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Event, attributes(discriminator))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let discriminator = input.attrs.iter()
        .find(|attr| attr.path().is_ident("discriminator"))
        .and_then(|attr| attr.parse_args::<Lit>().ok())
        .and_then(|lit| if let Lit::Int(lit_int) = lit {
            Some(lit_int.base10_parse::<u64>().unwrap())
        } else {
            None
        })
        .expect("Expected #[discriminator(u64)] attribute");

    let expanded = quote! {
        impl #impl_generics Event for #name #type_generics #where_clause {
            const DISCRIMINATOR: EventDiscriminator = EventDiscriminator::new(#discriminator);
        }
    };

    TokenStream::from(expanded)
}
