use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields};

mod structs;

#[proc_macro_derive(Jsonable)]
pub fn derive_jsonable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => {
            match structs::implement_named(&input.ident, fields) {
                Ok(output) => output,
                Err(err) => panic!("{}", err)
            }
        },
        Data::Struct(DataStruct { fields: Fields::Unit, .. }) => structs::implement_unit(&input.ident),
        Data::Struct(DataStruct { fields: Fields::Unnamed(fields), .. }) => {
            match structs::implement_unnamed(&input.ident, fields) {
                Ok(output) => output,
                Err(err) => panic!("{}", err)
            }
        },
        Data::Enum(_) => panic!("Unimplemented"),
        Data::Union(_) => panic!("Jsonable does not support unions")
    }.into()
}