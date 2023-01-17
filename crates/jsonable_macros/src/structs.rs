use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{FieldsNamed, FieldsUnnamed, Type};

pub fn implement_named(identifier: &Ident, input: FieldsNamed) -> Result<TokenStream, String> {
    let mut from_json_unchecked: Vec<TokenStream> = Vec::new();
    let mut to_json: Vec<TokenStream> = Vec::new();
    let mut validate_json: Vec<TokenStream> = Vec::new();

    for field in input.named.into_iter() {
        let ident = field.ident.unwrap();
        let ident_str = ident.to_string();
        let ty = field.ty;

        match ty.clone() {
            Type::Path(path) => {
                let complex =
                    path.path
                        .segments
                        .into_iter()
                        .find(|segment| match segment.arguments {
                            syn::PathArguments::None => false,
                            _ => true,
                        });
                if let Some(_) = complex {
                    from_json_unchecked.push(quote! {
                        #ident: <#ty as jsonable::Jsonable>::from_json_unchecked(inner_json.remove(#ident_str).unwrap_or(serde_json::Value::Null)),
                    });

                    validate_json.push(quote!{
                        match <#ty as jsonable::Jsonable>::validate_json(map.get(#ident_str).unwrap_or(&serde_json::Value::Null)) {
                            Ok(()) => (),
                            Err(err) => return Err(jsonable::JsonableError::InnerErrorForType { ty: std::any::type_name::<#ty>(), error: Box::from(err)})
                        }
                    });
                } else {
                    from_json_unchecked.push(quote! {
                        #ident: #ty::from_json_unchecked(inner_json.remove(#ident_str).unwrap_or(serde_json::Value::Null)),
                    });

                    validate_json.push(quote!{
                        match #ty::validate_json(map.get(#ident_str).unwrap_or(&serde_json::Value::Null)) {
                            Ok(()) => (),
                            Err(err) => return Err(jsonable::JsonableError::InnerErrorForType { ty: std::any::type_name::<#ty>(), error: Box::from(err)})
                        }
                    });
                }
            }
            _ => {
                from_json_unchecked.push(quote! {
                    #ident: <#ty as jsonable::Jsonable>::from_json_unchecked(inner_json.remove(#ident_str).unwrap_or(serde_json::Value::Null)),
                });

                validate_json.push(quote!{
                    match <#ty as jsonable::Jsonable>::validate_json(map.get(#ident_str).unwrap_or(&serde_json::Value::Null)) {
                        Ok(()) => (),
                        Err(err) => return Err(jsonable::JsonableError::InnerErrorForType { ty: std::any::type_name::<#ty>(), error: Box::from(err)})
                    }
                });
            }
        };

        to_json.push(quote! {
            map.insert(#ident_str.into(), self.#ident.to_json());
        });
    }

    let ident_str = identifier.to_string();

    Ok(quote! {
        impl jsonable::Jsonable for #identifier {
            fn from_json_unchecked(mut json: serde_json::Value) -> Self {
                let mut inner_json = json
                    .as_object_mut()
                    .unwrap_or_else(|| panic!("Tried converting non-object json to {}", #ident_str));
                Self {
                    #(#from_json_unchecked)*
                }
            }

            fn to_json(&self) -> serde_json::Value {
                let mut map = serde_json::Map::new();

                #(#to_json)*

                serde_json::Value::Object(map)
            }

            fn validate_json(json: &serde_json::Value) -> jsonable::Result<()> {
                match json {
                    serde_json::Value::Object(map) => {
                        #(#validate_json)*

                        Ok(())
                    },
                    serde_json::Value::Array(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "array", expected: "object" }),
                    serde_json::Value::Bool(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "bool", expected: "object" }),
                    serde_json::Value::Null => Err(jsonable::JsonableError::IncompatibleJsonType { got: "null", expected: "object" }),
                    serde_json::Value::Number(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "number", expected: "object" }),
                    serde_json::Value::String(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "string", expected: "object" })
                }
            }
        }
    })
}

pub fn implement_unnamed(identifier: &Ident, input: FieldsUnnamed) -> Result<TokenStream, String> {
    let mut from_json_unchecked: Vec<TokenStream> = Vec::new();
    let mut to_json: Vec<TokenStream> = Vec::new();
    let mut validate_json: Vec<TokenStream> = Vec::new();

    for (idx, field) in input.unnamed.into_iter().enumerate() {
        let ident_str = idx.to_string();
        let ty = field.ty;

        let index = syn::Index::from(idx);

        match ty.clone() {
            Type::Path(path) => {
                let complex =
                    path.path
                        .segments
                        .into_iter()
                        .find(|segment| match segment.arguments {
                            syn::PathArguments::None => false,
                            _ => true,
                        });
                if let Some(_) = complex {
                    from_json_unchecked.push(quote! {
                        #index: <#ty as jsonable::Jsonable>::from_json_unchecked(inner_json.remove(#ident_str).unwrap_or(serde_json::Value::Null)),
                    });

                    validate_json.push(quote!{
                        match <#ty as jsonable::Jsonable>::validate_json(map.get(#ident_str).unwrap_or(&serde_json::Value::Null)) {
                            Ok(()) => (),
                            Err(err) => return Err(jsonable::JsonableError::InnerErrorForType { ty: std::any::type_name::<#ty>(), error: Box::from(err)})
                        }
                    });
                } else {
                    from_json_unchecked.push(quote! {
                        #index: #ty::from_json_unchecked(inner_json.remove(#ident_str).unwrap_or(serde_json::Value::Null)),
                    });

                    validate_json.push(quote!{
                        match #ty::validate_json(map.get(#ident_str).unwrap_or(&serde_json::Value::Null)) {
                            Ok(()) => (),
                            Err(err) => return Err(jsonable::JsonableError::InnerErrorForType { ty: std::any::type_name::<#ty>(), error: Box::from(err)})
                        }
                    });
                }
            }
            _ => {
                from_json_unchecked.push(quote! {
                    #index: <#ty as jsonable::Jsonable>::from_json_unchecked(inner_json.remove(#ident_str).unwrap_or(serde_json::Value::Null)),
                });

                validate_json.push(quote!{
                    match <#ty as jsonable::Jsonable>::validate_json(map.get(#ident_str).unwrap_or(&serde_json::Value::Null)) {
                        Ok(()) => (),
                        Err(err) => return Err(jsonable::JsonableError::InnerErrorForType { ty: std::any::type_name::<#ty>(), error: Box::from(err)})
                    }
                });
            }
        };

        to_json.push(quote! {
            map.insert(#ident_str.into(), self.#index.to_json());
        });
    }

    let ident_str = identifier.to_string();

    Ok(quote! {
        impl jsonable::Jsonable for #identifier {
            fn from_json_unchecked(mut json: serde_json::Value) -> Self {
                let mut inner_json = json
                    .as_object_mut()
                    .unwrap_or_else(|| panic!("Tried converting non-object json to {}", #ident_str));
                Self {
                    #(#from_json_unchecked)*
                }
            }

            fn to_json(&self) -> serde_json::Value {
                let mut map = serde_json::Map::new();

                #(#to_json)*

                serde_json::Value::Object(map)
            }

            fn validate_json(json: &serde_json::Value) -> jsonable::Result<()> {
                match json {
                    serde_json::Value::Object(map) => {
                        #(#validate_json)*

                        Ok(())
                    },
                    serde_json::Value::Array(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "array", expected: "object" }),
                    serde_json::Value::Bool(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "bool", expected: "object" }),
                    serde_json::Value::Null => Err(jsonable::JsonableError::IncompatibleJsonType { got: "null", expected: "object" }),
                    serde_json::Value::Number(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "number", expected: "object" }),
                    serde_json::Value::String(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "string", expected: "object" })
                }
            }
        }
    })
}

pub fn implement_unit(identifier: &Ident) -> TokenStream {
    let ident_str = identifier.to_string();
    quote! {
        impl jsonable::Jsonable for #identifier {
            fn from_json_unchecked(json: serde_json::Value) -> Self {
                let inner_json = json
                    .as_object()
                    .unwrap_or_else(|| panic!("Tried converting non-object json to {}", #ident_str));;
                Self
            }

            fn to_json(&self) -> serde_json::Value {
                serde_json::Value::Object(serde_json::Map::new())
            }

            fn validate_json(json: &serde_json::Value) -> jsonable::Result<()> {
                match json {
                    serde_json::Value::Object(_) => Ok(()),
                    serde_json::Value::Array(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "array", expected: "object" }),
                    serde_json::Value::Bool(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "bool", expected: "object" }),
                    serde_json::Value::Null => Err(jsonable::JsonableError::IncompatibleJsonType { got: "null", expected: "object" }),
                    serde_json::Value::Number(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "number", expected: "object" }),
                    serde_json::Value::String(_) => Err(jsonable::JsonableError::IncompatibleJsonType { got: "string", expected: "object" })
                }
            }
        }
    }
}
