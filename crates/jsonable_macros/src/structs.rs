use proc_macro2::{TokenStream, Ident};
use quote::quote;
use syn::{FieldsNamed, Type};

pub fn implement_named(identifier: &Ident, input: FieldsNamed) -> Result<TokenStream, String> {
    let mut from_json_unchecked: Vec<TokenStream> = Vec::new();
    let mut to_json: Vec<TokenStream> = Vec::new();
    let mut verify_json: Vec<TokenStream> = Vec::new();

    for field in input.named.into_iter() {
        let ident = field.ident.unwrap();
        let ident_str = ident.to_string();
        let ty = field.ty;

        match ty.clone() {
            Type::Path(path) => {
                let complex = path.path.segments.into_iter().find(|segment| {
                    match segment.arguments {
                        syn::PathArguments::None => false,
                        _ => true
                    }
                });
                if let Some(_) = complex {
                    from_json_unchecked.push(quote! {
                        #ident: <#ty as jsonable::Jsonable>::from_json_unchecked(inner_json.remove(#ident_str).unwrap_or(serde_json::Value::Null)),
                    });

                    verify_json.push(quote!{
                        match <#ty as jsonable::Jsonable>::verify_json(map.get(#ident_str).unwrap_or(&serde_json::Value::Null)) {
                            Ok(()) => (),
                            Err(err) => return Err(jsonable::JsonableError::InnerErrorForType { ty: std::any::type_name::<#ty>(), error: Box::from(err)})
                        }
                    });
                } else {
                    if ident_str == "complex" {
                        panic!("Simple");
                    }
                    from_json_unchecked.push(quote! {
                        #ident: #ty::from_json_unchecked(inner_json.remove(#ident_str).unwrap_or(serde_json::Value::Null)),
                    });

                    verify_json.push(quote!{
                        match #ty::verify_json(map.get(#ident_str).unwrap_or(&serde_json::Value::Null)) {
                            Ok(()) => (),
                            Err(err) => return Err(jsonable::JsonableError::InnerErrorForType { ty: std::any::type_name::<#ty>(), error: Box::from(err)})
                        }
                    });
                }

            },
            _ => {
                if ident_str == "complex" {
                    panic!("Complex");
                }
                from_json_unchecked.push(quote! {
                    #ident: <#ty as jsonable::Jsonable>::from_json_unchecked(inner_json.remove(#ident_str).unwrap_or(serde_json::Value::Null)),
                });

                verify_json.push(quote!{
                    match <#ty as jsonable::Jsonable>::verify_json(map.get(#ident_str).unwrap_or(&serde_json::Value::Null)) {
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

    Ok(quote! {
        impl jsonable::Jsonable for #identifier {
            fn from_json_unchecked(mut json: serde_json::Value) -> Self {
                let mut inner_json = json.as_object_mut().unwrap();
                Self {
                    #(#from_json_unchecked)*
                }
            }

            fn to_json(&self) -> serde_json::Value {
                let mut map = serde_json::Map::new();

                #(#to_json)*

                serde_json::Value::Object(map)
            }

            fn verify_json(json: &serde_json::Value) -> jsonable::JsonResult<()> {
                match json {
                    serde_json::Value::Object(map) => {
                        #(#verify_json)*

                        Ok(())
                    }
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