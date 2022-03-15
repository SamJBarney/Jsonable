use proc_macro2::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FieldsNamed, FieldsUnnamed, Data, DataStruct, Fields};

mod structs;

#[proc_macro_derive(Jsonable)]
pub fn derive_jsonable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), ..}) => {
            match structs::implement_named(&input.ident, fields) {
                Ok(output) => output,
                Err(err) => panic!("{}", err)
            }
        }
        _ => panic!("this derive macro only works on structs with named fields"),
    }.into()
}