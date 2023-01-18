use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Fields, FieldsNamed, FieldsUnnamed, Variant};

pub fn implement(
    identifier: &Ident,
    variants: Punctuated<Variant, Comma>,
) -> Result<TokenStream, String> {
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

        match fields {
            Fields::Named(named_fields) => {
                let (mut validate, mut to, mut from_unchecked) =
                    match implement_named(&identifier_string, &ident, &ident_str, named_fields) {
                        Ok(result) => result,
                        Err(reason) => return Err(reason),
                    };
                validate_json_object.append(&mut validate);
                to_json.append(&mut to);
                from_json_unchecked_object.append(&mut from_unchecked);
            }
            Fields::Unnamed(unnamed_fields) => {
                let (mut validate, mut to, mut from_unchecked) =
                    match implement_unnamed(&identifier_string, &ident, &ident_str, unnamed_fields)
                    {
                        Ok(result) => result,
                        Err(reason) => return Err(reason),
                    };
                validate_json_object.append(&mut validate);
                to_json.append(&mut to);
                from_json_unchecked_object.append(&mut from_unchecked);
            }
            Fields::Unit => {
                validate_json_string.push(quote! {#ident_str => Ok(())});
                from_json_unchecked_string.push(quote! {#ident_str => Self::#ident});
                expected_string_types.push(ident_str.clone());
                to_json
                    .push(quote! { Self::#ident => serde_json::Value::String(#ident_str.into())});
            }
        }
    }
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
                    serde_json::Value::Object(mut map) => {
                        match map.keys().last().unwrap().as_str() {
                            #(#from_json_unchecked_object,)*
                            other => panic!("Unknown variant of enum '{}': {}", #identifier_string, other)
                        }
                    }
                    _ => panic!("Incompatible json for type '{}': {}", #identifier_string, json)
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
                        if map.len() == 1 {
                            let mut has_key = false;
                            #(#validate_json_object)*

                            if !has_key {
                                Err(jsonable::JsonableError::IncorrectKeyForEnum { ty: #identifier_string, key: map.keys().last().unwrap().clone() })
                            } else {
                                Ok(())
                            }
                        } else {
                            Err(jsonable::JsonableError::IncorrectObjectKeyCountForEnum {ty: #identifier_string, count: map.len() })
                        }
                    },
                    serde_json::Value::String(value) => {
                        match value.as_str() {
                            #(#validate_json_string,)*
                            other => Err(jsonable::JsonableError::InvalidEnumStringVariant { enum_type: #identifier_string, got: value.clone(), expected: vec![#(#expected_string_types,)*]})
                        }
                    },
                    serde_json::Value::Array(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "array", expected: "object or string" }),
                    serde_json::Value::Bool(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "bool", expected: "object or string" }),
                    serde_json::Value::Null => Err(jsonable::JsonableError::IncompatibleJsonType { got: "null", expected: "object or string" }),
                    serde_json::Value::Number(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "number", expected: "object or string" }),
                }
            }
        }
    })
}

fn implement_named(
    type_ident_str: &String,
    ident: &Ident,
    ident_str: &String,
    fields: FieldsNamed,
) -> Result<(Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>), String> {
    let mut validate = Vec::new();
    let mut to_json = Vec::new();
    let mut from_unchecked = Vec::new();
    let named = fields.named;
    let field_count = named.len();

    let mut validate_parts = Vec::new();
    let mut to_json_parts = Vec::new();
    let mut from_unchecked_parts = Vec::new();

    let mut field_idents: Vec<Ident> = Vec::with_capacity(field_count);

    for field in named {
        let ty = field.ty;
        let field_ident = field.ident.unwrap();
        let field_ident_str = field_ident.to_string();

        from_unchecked_parts.push(quote!{
            #field_ident: if let Some(value) = inner_map.remove(#field_ident_str) { <#ty as jsonable::Jsonable>::from_json_unchecked(value) } else { panic!("Missing field '{}' for variant `{}::{}`", #field_ident_str, #type_ident_str, #ident_str) }
        });

        to_json_parts
            .push(quote! {inner_map.insert(#field_ident_str.into(), #field_ident.to_json());});

        validate_parts.push(quote!{
            if let Some(value) = inner_map.get(#field_ident_str) {
                match <#ty as jsonable::Jsonable>::validate_json(value) {
                    Ok(_) => {},
                    Err(err) => errors.push(jsonable::JsonableError::InnerErrorForType {ty: std::any::type_name::<#ty>(), error: err.into()})
                }
            } else {
                errors.push(jsonable::JsonableError::MissingKeyForEnumVariant {variant: #ident_str, key: #field_ident_str});
            }
        });

        field_idents.push(field_ident);
    }

    from_unchecked.push(quote!{
        #ident_str => {
            if let Some(inner_map) = map.remove(#ident_str).unwrap().as_object_mut() {
                Self::#ident{#(#from_unchecked_parts,)*}
            } else {
                panic!("Attempted converting non-object to enum variant `{}::{}`", #type_ident_str, #ident_str)
            }
        }
    });

    to_json.push(quote!{
        Self::#ident {#(#field_idents,)*} => {
            let mut inner_map = serde_json::Map::with_capacity(#field_count);

            #(#to_json_parts)*

            serde_json::Value::Object(serde_json::Map::from_iter([(#ident_str.into(), serde_json::Value::Object(inner_map))]))
        }
    });

    validate.push(quote!{
        if !has_key && map.contains_key(#ident_str) {
            has_key = true;

            if let Some(inner_map) = map.get(#ident_str).unwrap().as_object() {
                if inner_map.len() == #field_count {
                    let mut errors = Vec::new();

                    #(#validate_parts)*

                    if errors.len() > 0 {
                        return Err(jsonable::JsonableError::InnerErrorsForType {ty: #type_ident_str, errors })
                    } else {
                        return Ok(())
                    }
                } else {
                    return Err(jsonable::JsonableError::IncorrectFieldCountForEnum {enum_type: #type_ident_str, variant: #ident_str, count: #field_count})
                }
            } else {
                return Err(jsonable::JsonableError::IncompatibleJsonType {got: "other", expected: "object"})
            }
        }
    });

    Ok((validate, to_json, from_unchecked))
}

fn implement_unnamed(
    type_ident_str: &String,
    ident: &Ident,
    ident_str: &String,
    fields: FieldsUnnamed,
) -> Result<(Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>), String> {
    let mut validate: Vec<TokenStream> = Vec::new();
    let mut to_json: Vec<TokenStream> = Vec::new();
    let mut from_unchecked: Vec<TokenStream> = Vec::new();
    let unnamed = fields.unnamed;
    let count = unnamed.len();

    if count > 1 {
        let mut validate_parts: Vec<TokenStream> = Vec::with_capacity(count);
        let mut to_json_parts: Vec<TokenStream> = Vec::with_capacity(count);
        let mut from_unchecked_parts: Vec<TokenStream> = Vec::with_capacity(count);
        for (idx, field) in unnamed.iter().enumerate() {
            let ty = field.ty.clone();
            from_unchecked_parts.push(quote! {
                <#ty as jsonable::Jsonable>::from_json_unchecked(array.pop().unwrap())
            });

            validate_parts.push(quote!{
                match <#ty as jsonable::Jsonable>::validate_json(array.get(#idx).unwrap()) {
                    Ok(_) => {},
                    Err(err) => errors.push(jsonable::JsonableError::InnerErrorForType {ty: std::any::type_name::<#ty>(), error: err.into()})
                };
            });

            let field_name = Ident::new(format!("field{}", idx).as_str(), ident.span());

            to_json_parts.push(quote! {
                array.push(#field_name.to_json())
            });
        }

        from_unchecked.push(quote!{
            #ident_str => {
                if let Some(array) = map.remove(#ident_str).unwrap().as_array_mut() {
                    if array.len() == #count {
                        Self::#ident(#(#from_unchecked_parts,)*)
                    } else {
                        panic!("Unexpected array length for enum varient '{}::{}'. Got {}, expected {}", #type_ident_str, #ident_str, array.len(), #count)
                    }
                } else {
                    panic!("Tried converting non-array json to enum variant `{}::{}`", #type_ident_str, #ident_str)
                }
            }
        });

        validate.push(quote! {
            if !has_key && map.contains_key(#ident_str) {
                has_key = true;
                if let Some(array) = map.get(#ident_str).unwrap().as_array() {
                    if array.len() == #count {
                        let mut errors = Vec::with_capacity(#count);
                        #(#validate_parts)*
                        if errors.len() > 0 {
                            return Err(jsonable::JsonableError::InnerErrorsForType { ty: #type_ident_str, errors})
                        } else {
                            return Ok(())
                        }
                    } else {
                        return Err(jsonable::JsonableError::IncorrectFieldCountForEnum{ enum_type: #type_ident_str, variant: #ident_str, count: #count})
                    }
                } else {
                    return Err(jsonable::JsonableError::IncompatibleJsonType {got: "other", expected: "array"})
                }
            }
        });

        let fields: Vec<Ident> = (0..count)
            .map(|idx| Ident::new(format!("field{}", idx).as_str(), ident.span()))
            .collect();
        to_json.push(quote! {
            Self::#ident(#(#fields,)*) => {
                let mut array = Vec::with_capacity(#count);

                #(#to_json_parts;)*

                serde_json::Value::Array(array)
            }
        });
    } else {
        let field = unnamed.first().unwrap().clone();
        let ty = field.ty;
        from_unchecked.push(quote!{
            #ident_str => {
                Self::#ident( <#ty as jsonable::Jsonable>::from_json_unchecked(map.remove(#ident_str).unwrap_or(serde_json::Value::Null)) )
            }
        });
        validate.push(quote! {
            if !has_key && map.contains_key(#ident_str) {
                has_key = true;
                let inner_json = map.get(#ident_str).unwrap();
                match <#ty as jsonable::Jsonable>::validate_json(inner_json) {
                    Ok(_) => {},
                    Err(err) => return Err(
                        jsonable::JsonableError::InnerErrorForType{ ty: #ident_str, error: jsonable::JsonableError::InnerErrorForType{ ty: std::any::type_name::<#ty>(),  error: err.into() }.into()}
                    )
                };
            }
        });
        to_json.push(quote!{Self::#ident(field1) => serde_json::Value::Object(serde_json::Map::from_iter([ (String::from(#ident_str), field1.to_json())])) });
    }

    Ok((validate, to_json, from_unchecked))
}
