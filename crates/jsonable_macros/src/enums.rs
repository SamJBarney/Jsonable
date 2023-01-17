use syn::{punctuated::Punctuated, Variant, token::Comma, Fields, FieldsUnnamed};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn implement(identifier: &Ident, variants: Punctuated<Variant, Comma>) -> Result<TokenStream, String> {
    let identifier_string = identifier.to_string();
    let mut from_json_unchecked_string: Vec<TokenStream> = Vec::new();
    let mut from_json_unchecked_object: Vec<TokenStream> = Vec::new();
    let mut to_json: Vec<TokenStream> = Vec::new();
    let mut validate_json_string: Vec<TokenStream> = Vec::new();
    let mut validate_json_object: Vec<TokenStream> = Vec::new();
    let mut expected_string_types: Vec<String> = Vec::new();

    for variant in variants.into_iter() {
        let ident = variant.ident;
        let ident_str = ident.to_string();
        let fields = variant.fields;

        // If there are associated fields, then 
        if fields.len() > 0 {
            match fields {
                Fields::Named(named_fields) => {},
                Fields::Unnamed(unnamed_fields) => {
                    let (mut validate, mut to, mut from_unchecked) = match implement_unnamed(&ident, &ident_str, unnamed_fields) {
                        Ok(result) => result,
                        Err(reason) => return Err(reason)
                    };
                    validate_json_object.append(&mut validate);
                    to_json.append(&mut to);
                    from_json_unchecked_object.append(&mut from_unchecked);
                },
                Fields::Unit => return Err(format!("Unit field for enum variant '{}::{}'. How is this possible!?", identifier_string, ident_str))
            }
        } else {
            validate_json_string.push(quote! {#ident_str => Ok(())});
            from_json_unchecked_string.push(quote! {#ident_str => Self::#ident});
            expected_string_types.push(ident_str.clone());
            to_json.push(quote! { Self::#ident => serde_json::Value::String(#ident_str.into())});
        }
    };
    Ok(quote! {
        impl jsonable::Jsonable for #identifier {
            fn from_json_unchecked(mut json: serde_json::Value) -> Self {
                match json {
                    serde_json::Value::String(value) => {
                        match value.as_str() {
                            #(#from_json_unchecked_string,)*
                            other => panic!("Unknown variant of enum '{}': {}", #identifier_string, value)
                        }
                    },
                    _ => #identifier::ASDF
                }
            }

            fn to_json(&self) -> serde_json::Value {
                match self {
                    #(#to_json,)*
                }
            }

            fn validate_json(json: &serde_json::Value) -> jsonable::Result<()> {
                match json {
                    serde_json::Value::Object(map) => {
                        #(#validate_json_object)*

                        Ok(())
                    },
                    serde_json::Value::String(value) => {
                        match value.as_str() {
                            #(#validate_json_string,)*
                            other => Err(jsonable::JsonableError::InvalidEnumStringVariant { enum_type: #identifier_string, got: value.clone(), expected: vec![#(#expected_string_types,)*]})
                        }
                    },
                    serde_json::Value::Array(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "array", expected: "object" }),
                    serde_json::Value::Bool(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "bool", expected: "object" }),
                    serde_json::Value::Null => Err(jsonable::JsonableError::IncompatibleJsonType { got: "null", expected: "object" }),
                    serde_json::Value::Number(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "number", expected: "object" }),
                }
            }
        }
    })
}


fn implement_unnamed(ident: &Ident, ident_str: &String, fields: FieldsUnnamed) -> Result<(Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>), String> {
    let mut validate = Vec::new();
    let mut to_json = Vec::new();
    let mut from_unchecked = Vec::new();

    Ok((validate, to_json, from_unchecked))
}