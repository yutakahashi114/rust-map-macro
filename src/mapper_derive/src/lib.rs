extern crate proc_macro;
extern crate syn;
use anyhow::{anyhow, Result};
use syn::Data;
use syn::Fields;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Mapper)]
pub fn derive_mapper(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    impl_mapper_macro(&input).unwrap()
}

fn impl_mapper_macro(input: &syn::DeriveInput) -> Result<TokenStream> {
    let data_struct = match &input.data {
        Data::Struct(data_struct) => data_struct,
        Data::Enum(_) => Err(anyhow!("invalid type: Enum"))?,
        Data::Union(_) => Err(anyhow!("invalid type: Union"))?,
    };

    let fields_named = match &data_struct.fields {
        Fields::Named(fields_named) => fields_named,
        Fields::Unnamed(_) => Err(anyhow!("invalid type: Unnamed"))?,
        Fields::Unit => Err(anyhow!("invalid type: Unit"))?,
    };

    let to_field_value_token_streams: Vec<proc_macro2::TokenStream> = fields_named
        .named
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None => syn::Member::Unnamed(i.into()),
            };
            return quote! {
                result.insert(stringify!(#name).to_string(), rust_map_macro::mapper::Converter::to_field_value(&self.#name));
            };
        })
        .collect();

    let to_primitive_token_streams: Vec<proc_macro2::TokenStream> = fields_named
        .named
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None => syn::Member::Unnamed(i.into()),
            };
            let ty = &field.ty;
            return quote! {
                let mut #name: Option<#ty> = None;
                if let Some(value) = __optional_map__.get_mut(stringify!(#name)) {
                    if let Some(value) = std::mem::replace(value, None) {
                        #name = Some(rust_map_macro::mapper::Converter::to_primitive(value)?);
                    } else {
                        return Err(anyhow::anyhow!("invalid type: {}", stringify!(#ty)));
                    }
                }
                let #name = #name.ok_or(anyhow::anyhow!("invalid type: {}", stringify!(#ty)))?;
            };
        })
        .collect();

    let to_struct_token_streams: Vec<proc_macro2::TokenStream> = fields_named
        .named
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None => syn::Member::Unnamed(i.into()),
            };
            return quote! {
                #name,
            };
        })
        .collect();

    let name = &input.ident;
    let (im_generics, ty_generics, _) = input.generics.split_for_impl();

    Ok(quote! {
        impl#im_generics Mapper for #name#ty_generics {
            fn to_map(&self) -> std::collections::HashMap<String, rust_map_macro::mapper::FieldValue> {
                let mut result = std::collections::HashMap::new();
                #(#to_field_value_token_streams)*
                result
            }

            fn from_map(__map__: std::collections::HashMap<String, rust_map_macro::mapper::FieldValue>) -> anyhow::Result<Self> {
                let mut __optional_map__ = std::collections::HashMap::with_capacity(__map__.len());
                for (key, val) in __map__ {
                    __optional_map__.insert(key, Some(val));
                }
                #(#to_primitive_token_streams)*
                Ok(#name { #(#to_struct_token_streams)* })
            }
        }
    }
    .into())
}
