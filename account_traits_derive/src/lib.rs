use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(AccountDeserialize)]
pub fn derive_account_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl AccountDeserialize for #name {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ToBytes)]
pub fn derive_to_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl ToBytes for #name {
            fn to_bytes(&self) -> &[u8] {
                bytemuck::bytes_of(self)
            }
        }
    };

    TokenStream::from(expanded)
}
