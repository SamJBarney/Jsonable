use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use serde_json::{self, Map, Value};

/// Error enum returned from [Jsonable::from_json] or [Jsonable::validate_json]
///
/// `IncompatibleJsonType` - json cannot be converted to the current type
///
/// `IncompatibleEntryForType` - json array contains a value that cannot be converted to the current type
///
/// `InnerErrorForType` - at least one json object's value cannot be converted to its type
///
/// ## Examples
/// ```ignore
/// use serde_json::{Result, Value};
/// use jsonable::Jsonable;
///
/// // IncompatibleJsonType
/// fn incompatible_example() -> Result<()> {
///     let v: Value = serde_json::from_str(r#"{
///         "name": {
///             "en_US": "A Research Of Papers",
///             "es-MX": "Una investigación de artículos"
///         },
///         "pages": []
///     }"#)?;
///
///     match Vec::<String>::from_json(v) {
///         Ok(_) => println!("This shouldn't have worked! How did you get here!?"),
///         Err(err) => println!("Got an error validating json: {:?}", err)
///     };
///
///     Ok(())
/// }
///
/// // IncompatibleJsonType
/// fn incompatible_entry_example() -> Result<()> {
///     let v: Value = serde_json::from_str(r#"[
///         "Why are you reading this?",
///         13,
///         "No, there isn't a hidden message."
///     ]"#)?;
///
///     match Vec::<String>::from_json(v) {
///         Ok(_) => println!("This shouldn't have worked! How did you get here!?"),
///         Err(err) => println!("Got an error validating json: {:?}", err)
///     };
///
///     Ok(())
/// }
///
///
/// #[derive(Jsonable)]
/// struct ResearchPaper {
///     name: String,
///     pages: Vec<String>
/// }
///
/// // InnerErrorForType
/// fn inner_error_example() -> Result<()> {
///      let v: Value = serde_json::from_str(r#"{
///         "name": "Around the Riverbend: A Study of River Ecosystems",
///         "pages": [
///             "I don't actually know anything about river ecosystems, however",
///             12,
///             "and if that doesn't convince you, I don't know what will."
///         ]
///     }"#)?;
///
///     match ResearchPaper::from_json(v) {
///         Ok(_) => println!("This shouldn't have worked! How did you get here!?"),
///         Err(err) => println!("Got an error validating json: {:?}", err)
///     };
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum JsonableError {
    IncompatibleJsonType {
        got: &'static str,
        expected: &'static str,
    },
    IncompatibleEntryForType(&'static str),
    InnerErrorForType {
        ty: &'static str,
        error: Box<JsonableError>,
    },
    InvalidArrayLength {
        got: usize,
        expected: usize,
    },
}

/// Return type for [Jsonable::from_json] and [Jsonable::validate_json]
pub type Result<T> = core::result::Result<T, JsonableError>;

/// A **data structure** that can be converted to and from [serde_json::Value](https://docs.serde.rs/serde_json/value/enum.Value.html).
pub trait Jsonable: Sized {
    /// Consumes the [serde_json::Value](https://docs.serde.rs/serde_json/value/enum.Value.html) and returns the resulting value unless validation failed.
    /// Provides a default implementation.
    fn from_json(json: Value) -> Result<Self> {
        match Self::validate_json(&json) {
            Ok(_) => Ok(Self::from_json_unchecked(json)),
            Err(err) => Err(err),
        }
    }

    /// Consumes the [serde_json::Value](https://docs.serde.rs/serde_json/value/enum.Value.html) and returns the resulting value.
    /// Provided implementations panic if conversion failed.
    fn from_json_unchecked(json: Value) -> Self;

    /// Converts the object into a [serde_json::Value](https://docs.serde.rs/serde_json/value/enum.Value.html).
    fn to_json(&self) -> Value;

    /// Validates that the provided [serde_json::Value](https://docs.serde.rs/serde_json/value/enum.Value.html) can be converted to the type.
    fn validate_json(json: &Value) -> Result<()>;
}

impl<T: Jsonable> Jsonable for Vec<T> {
    /// Panics if the [serde_json::Value](https://docs.serde.rs/serde_json/value/enum.Value.html) is not an [Array](https://docs.serde.rs/serde_json/value/enum.Value.html#variant.Array)
    fn from_json_unchecked(mut json: Value) -> Self {
        json.as_array_mut()
            .unwrap_or_else(|| panic!("Tried converting non-array json to Vec"))
            .to_owned()
            .into_iter()
            .map(|value| T::from_json_unchecked(value))
            .collect::<Self>()
    }

    fn to_json(&self) -> Value {
        Value::Array(self.into_iter().map(|entry| entry.to_json()).collect())
    }
    /// Returns `Ok(())` for an [Array](https://docs.serde.rs/serde_json/value/enum.Value.html#variant.Array).
    ///
    /// Returns Err([JsonableError::IncompatibleEntryForType]) if the entries in the array cannot be converted to T.
    ///
    /// Returns Err([JsonableError::IncompatibleJsonType]) if the json value is not an array.
    fn validate_json(json: &Value) -> Result<()> {
        match json {
            Value::Array(vec) => {
                if vec.into_iter().all(|entry| match T::validate_json(&entry) {
                    Ok(_) => true,
                    Err(_) => false,
                }) {
                    Ok(())
                } else {
                    Err(JsonableError::IncompatibleEntryForType(
                        std::any::type_name::<T>(),
                    ))
                }
            }
            Value::Bool(_) => Err(JsonableError::IncompatibleJsonType {
                got: "bool",
                expected: "array",
            }),
            Value::Null => Err(JsonableError::IncompatibleJsonType {
                got: "null",
                expected: "array",
            }),
            Value::Number(_) => Err(JsonableError::IncompatibleJsonType {
                got: "number",
                expected: "array",
            }),
            Value::Object(_) => Err(JsonableError::IncompatibleJsonType {
                got: "object",
                expected: "array",
            }),
            Value::String(_) => Err(JsonableError::IncompatibleJsonType {
                got: "string",
                expected: "array",
            }),
        }
    }
}

impl<I, T> Jsonable for HashMap<I, T>
where
    I: From<String> + Into<String> + Hash + Eq + Clone,
    T: Jsonable,
    String: From<I>,
{
    fn from_json_unchecked(json: Value) -> Self {
        let obj = json
            .as_object()
            .unwrap_or_else(|| panic!("Tried converting non-object json to HashMap"));
        let mut map = HashMap::with_capacity(obj.keys().len());
        for (key, value) in obj.into_iter() {
            map.insert(
                I::from(key.to_owned()),
                T::from_json_unchecked(value.to_owned()),
            );
        }

        map
    }

    fn to_json(&self) -> Value {
        let mut obj = Map::with_capacity(self.keys().len());
        for (key, value) in self.into_iter() {
            let k = key.clone().into();
            obj.insert(k, value.to_json());
        }

        Value::Object(obj)
    }

    fn validate_json(json: &Value) -> Result<()> {
        match json {
            Value::Object(map) => {
                if map.values().all(|value| match T::validate_json(value) {
                    Ok(()) => true,
                    _ => false,
                }) {
                    Ok(())
                } else {
                    Err(JsonableError::IncompatibleEntryForType(
                        std::any::type_name::<T>(),
                    ))
                }
            }
            Value::Array(_) => Err(JsonableError::IncompatibleJsonType {
                got: "array",
                expected: "object",
            }),
            Value::Bool(_) => Err(JsonableError::IncompatibleJsonType {
                got: "bool",
                expected: "object",
            }),
            Value::Null => Err(JsonableError::IncompatibleJsonType {
                got: "null",
                expected: "object",
            }),
            Value::Number(_) => Err(JsonableError::IncompatibleJsonType {
                got: "number",
                expected: "object",
            }),
            Value::String(_) => Err(JsonableError::IncompatibleJsonType {
                got: "string",
                expected: "object",
            }),
        }
    }
}

impl<T> Jsonable for HashSet<T>
where
    T: Jsonable + Eq + Hash,
{
    fn from_json_unchecked(mut json: Value) -> Self {
        let vec = json
            .as_array_mut()
            .unwrap_or_else(|| panic!("Tried converting non-array json into hashset"));
        let mut set = HashSet::with_capacity(vec.len());
        for value in vec.drain(..) {
            set.insert(T::from_json_unchecked(value));
        }

        set
    }

    fn to_json(&self) -> Value {
        let mut vec = Vec::new();

        for entry in self.into_iter() {
            vec.push(entry.to_json());
        }

        Value::Array(vec)
    }

    fn validate_json(json: &Value) -> Result<()> {
        Vec::<T>::validate_json(json)
    }
}

impl<T> Jsonable for Option<T>
where
    T: Jsonable,
{
    fn from_json_unchecked(json: Value) -> Self {
        match json {
            Value::Null => None,
            _ => Some(T::from_json_unchecked(json)),
        }
    }

    fn to_json(&self) -> Value {
        if let Some(value) = self {
            value.to_json()
        } else {
            Value::Null
        }
    }

    fn validate_json(json: &Value) -> Result<()> {
        match json {
            Value::Null => Ok(()),
            _ => T::validate_json(json),
        }
    }
}

impl Jsonable for String {
    fn from_json_unchecked(json: Value) -> Self {
        json.as_str()
            .unwrap_or_else(|| panic!("Tried converting non-string json into string"))
            .into()
    }

    fn to_json(&self) -> Value {
        Value::String(self.clone())
    }

    fn validate_json(json: &Value) -> Result<()> {
        match json {
            Value::String(_) => Ok(()),
            Value::Null => Err(JsonableError::IncompatibleJsonType {
                got: "null",
                expected: "string",
            }),
            Value::Bool(_) => Err(JsonableError::IncompatibleJsonType {
                got: "bool",
                expected: "string",
            }),
            Value::Number(_) => Err(JsonableError::IncompatibleJsonType {
                got: "number",
                expected: "string",
            }),
            Value::Array(_) => Err(JsonableError::IncompatibleJsonType {
                got: "array",
                expected: "string",
            }),
            Value::Object(_) => Err(JsonableError::IncompatibleJsonType {
                got: "object",
                expected: "string",
            }),
        }
    }
}

impl<T: Jsonable, const N: usize> Jsonable for [T; N] {
    fn from_json_unchecked(mut json: Value) -> Self {
        json.as_array_mut()
            .unwrap_or_else(|| panic!("Tried converting non-array json to fixed sized array"))
            .to_owned()
            .into_iter()
            .map(|value| T::from_json_unchecked(value))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|v: Vec<T>| {
                panic!("Expected Vec or length {}. Got {} instead", N, v.len())
            })
    }

    fn to_json(&self) -> Value {
        Value::Array(
            self.into_iter()
                .map(|value| value.to_json())
                .collect::<Vec<_>>(),
        )
    }

    fn validate_json(json: &Value) -> Result<()> {
        match json {
            Value::Array(arr) => {
                if arr.len() == N {
                    if arr.into_iter().all(|value| T::validate_json(value).is_ok()) {
                        Ok(())
                    } else {
                        Err(JsonableError::IncompatibleEntryForType(
                            std::any::type_name::<T>(),
                        ))
                    }
                } else {
                    Err(JsonableError::InvalidArrayLength {
                        got: arr.len(),
                        expected: N,
                    })
                }
            }
            Value::Null => Err(JsonableError::IncompatibleJsonType {
                got: "null",
                expected: "array",
            }),
            Value::String(_) => Err(JsonableError::IncompatibleJsonType {
                got: "string",
                expected: "array",
            }),
            Value::Bool(_) => Err(JsonableError::IncompatibleJsonType {
                got: "bool",
                expected: "array",
            }),
            Value::Number(_) => Err(JsonableError::IncompatibleJsonType {
                got: "number",
                expected: "array",
            }),
            Value::Object(_) => Err(JsonableError::IncompatibleJsonType {
                got: "object",
                expected: "array",
            }),
        }
    }
}

macro_rules! number_impl {
    ($ty: ty, $method: ident) => {
        impl Jsonable for $ty {
            fn from_json_unchecked(json: Value) -> Self {
                json.$method().unwrap_or_else(|| {
                    panic!(
                        "Tried converting non-number json to {}",
                        std::any::type_name::<$ty>()
                    )
                }) as $ty
            }

            fn to_json(&self) -> Value {
                Value::from(*self)
            }

            fn validate_json(json: &Value) -> Result<()> {
                match json {
                    Value::Number(_) => Ok(()),
                    Value::Array(_) => Err(JsonableError::IncompatibleJsonType {
                        got: "array",
                        expected: "number",
                    }),
                    Value::Bool(_) => Err(JsonableError::IncompatibleJsonType {
                        got: "bool",
                        expected: "number",
                    }),
                    Value::Null => Err(JsonableError::IncompatibleJsonType {
                        got: "null",
                        expected: "number",
                    }),
                    Value::Object(_) => Err(JsonableError::IncompatibleJsonType {
                        got: "object",
                        expected: "number",
                    }),
                    Value::String(_) => Err(JsonableError::IncompatibleJsonType {
                        got: "string",
                        expected: "number",
                    }),
                }
            }
        }
    };
}

number_impl!(u8, as_u64);
number_impl!(u16, as_u64);
number_impl!(u32, as_u64);
number_impl!(u64, as_u64);
number_impl!(i8, as_i64);
number_impl!(i16, as_i64);
number_impl!(i32, as_i64);
number_impl!(i64, as_i64);
number_impl!(f32, as_f64);
number_impl!(f64, as_f64);

#[cfg(test)]
pub mod tests {
    pub use super::*;
    pub use serde_json::*;

    // Enabled test module
    #[allow(unused_macros)]
    macro_rules! test_mod {
        ($name:ident { $( $rest:tt )* }) => {
            mod $name {
                pub use super::*;
                $($rest)*
            }
        };
    }

    // Disabled test module
    #[allow(unused_macros)]
    macro_rules! xtest_mod {
        ($name:ident { $( $rest:tt )* }) => {};
    }

    test_mod! { fixed_array {
        pub type Subject = [u8;4];

        test_mod!{ from_json_unchecked {
            #[test]
            fn happy_path() {
                let result = Subject::from_json_unchecked(json!([1,2,3,4]));
                assert_eq!(result, [1, 2, 3, 4]);
            }

            #[test]
            #[should_panic]
            fn incorrect_json_type() {
                Subject::from_json_unchecked(json!({}));
            }

            #[test]
            #[should_panic]
            fn incorrect_array_length() {
                Subject::from_json_unchecked(json!([1, 2, 3]));
            }
        }}

        test_mod!{ to_json {
            #[test]
            fn happy_path() {
                let subject: Subject = [1, 2, 3, 4];
                let json = subject.to_json();
                assert_eq!(json, json!([1, 2, 3, 4]));
            }
        }}

        test_mod!{ validate_json {
            #[test]
            fn happy_path() {
                assert!(Subject::validate_json(&json!([1,2,3,4])).is_ok());
            }

            #[test]
            fn incorrect_json_type() {
                match Subject::validate_json(&json!({})) {
                    Err(err) => {
                        assert_eq!{ err, JsonableError::IncompatibleJsonType { expected: "array", got: "object" } }
                    },
                    _ => assert!(false)
                };
            }

            #[test]
            fn incorrect_length() {
                match Subject::validate_json(&json!([1,2,3])) {
                    Err(err) => {
                        assert_eq!{ err, JsonableError::InvalidArrayLength { got: 3, expected: 4 } }
                    },
                    _ => assert!(false)
                };
            }
        }}
    }}

    test_mod! { hash_map {
        pub use std::collections::HashMap;
        pub type Subject = HashMap<String, u8>;

        test_mod!{ from_json_unchecked {
            #[test]
            fn happy_path() {
                let result = Subject::from_json_unchecked(json!({
                    "key": 1 as u8
                }));

                assert!(result.contains_key("key".into()));
                assert_eq!(result.get("key".into()), Some(&1));
            }

            #[test]
            #[should_panic]
            fn incorrect_json_type() {
                Subject::from_json_unchecked(json!([]));
            }
        }}

        test_mod!{ to_json {
            #[test]
            fn happy_path() {
                let mut subject: Subject = Subject::new();
                subject.insert("key".into(), 1);

                let json = subject.to_json();

                assert_eq!(json, json!({"key": 1}));
            }
        }}

        test_mod!{ validate_json {
            #[test]
            fn happy_path() {
                let result = Subject::validate_json(&json!({
                    "key": 1 as u8
                }));
                assert!(result.is_ok());
            }

            #[test]
            fn incorrect_json_type() {
                let result = Subject::validate_json(&json!([]));

                match result {
                    Err(err) => {
                        assert_eq!(err, JsonableError::IncompatibleJsonType { got: "array", expected: "object" })
                    },
                    _ => assert!(false)
                };
            }
        }}
    }}

    test_mod! {hash_set {
        pub use std::collections::HashSet;
        pub type Subject = HashSet<String>;

        test_mod!{ from_json_unchecked {
            #[test]
            fn happy_path() {
                let values: Vec<String> = vec!["Value 1".into(), "Value 2".into()];
                let json = Value::Array(values.clone().into_iter().map(|value| Value::String(value)).collect::<Vec<_>>());
                let subject = Subject::from_json_unchecked(json);

                assert_eq!(subject.len(), values.len());
                for value in values.iter() {
                    assert!(subject.contains(value));
                }
            }

            #[test]
            #[should_panic]
            fn incorrect_json_type() {
                Subject::from_json_unchecked(json!({}));
            }
        }}

        test_mod!{ to_json {
            #[test]
            fn happy_path() {
                let mut subject = Subject::new();
                subject.insert("Hello".into());
                subject.insert("World".into());

                let json = subject.to_json();

                // HashSet does not return keys in a consistent order
                // Assertions must not depend on order
                assert!(json.is_array());
                let vec = json.as_array().unwrap();
                assert!(vec.contains(&json!("Hello")));
                assert!(vec.contains(&json!("World")));
            }
        }}

        test_mod!{ validate_json {
            #[test]
            fn happy_path() {
                let values: Vec<String> = vec!["Value 1".into(), "Value 2".into()];
                let json = Value::Array(values.clone().into_iter().map(|value| Value::String(value)).collect::<Vec<_>>());

                assert!(Subject::validate_json(&json).is_ok());
            }

            #[test]
            fn incorrect_json_type() {
                let result = Subject::validate_json(&json!({}));

                match result {
                    Err(err) => {
                        assert_eq!(err, JsonableError::IncompatibleJsonType { got: "object", expected: "array" })
                    },
                    _ => assert!(false)
                };
            }
        }}
    }}

    test_mod! {option {
        pub type Subject = Option<u8>;

        test_mod!{ from_json_unchecked {
            #[test]
            fn happy_path() {
                let result = Subject::from_json_unchecked(json!(8));
                assert_eq!(result, Some(8 as u8));
            }
            #[test]
            fn happy_path_null() {
                let result = Subject::from_json_unchecked(json!(null));
                assert_eq!(result, None);
            }
        }}

        test_mod!{ to_json {
            #[test]
            fn happy_path() {
                let subject: Subject = Some(8);
                let result = subject.to_json();
                assert_eq!(result, json!(8));
            }

            #[test]
            fn happy_path_null() {
                let subject: Subject = None;
                let result = subject.to_json();
                assert_eq!(result, json!(null));
            }
        }}

        test_mod!{ validate_json {
            #[test]
            fn happy_path() {
                assert!(Subject::validate_json(&json!(8)).is_ok());
            }

            #[test]
            fn happy_path_null() {
                assert!(Subject::validate_json(&json!(null)).is_ok());
            }
        }}
    }}

    test_mod! { string {
        pub type Subject = String;

        test_mod!{ from_json_unchecked {
            #[test]
            fn happy_path() {
                let result: Subject = Subject::from_json_unchecked(json!("Uh huh"));
                assert_eq!(result, Subject::from("Uh huh"));
            }

            #[test]
            #[should_panic]
            fn incorrect_json_type() {
                Subject::from_json_unchecked(json!({}));
            }
        }}

        test_mod!{ to_json {
            #[test]
            fn happy_path() {
                let subject = Subject::from("This is a triumph; huge success.");
                let json = subject.to_json();

                assert_eq!(json, json!("This is a triumph; huge success."));
            }
        }}

        test_mod!{ validate_json {
            #[test]
            fn happy_path() {
                assert!(Subject::validate_json(&json!("I'm a string")).is_ok());
            }

            #[test]
            fn incorrect_json_type() {
                let result = Subject::validate_json(&json!({}));

                match result {
                    Err(err) => assert_eq!(err, JsonableError::IncompatibleJsonType { got: "object", expected: "string" }),
                    _ => assert!(false)
                };
            }
        }}
    }}

    test_mod! { vec {
        pub type Subject = Vec<u8>;

        test_mod!{ from_json_unchecked {
            #[test]
            fn happy_path() {
                let subject = Subject::from_json_unchecked(json!([1, 2, 3, 4]));

                assert_eq!(subject, vec![1, 2, 3, 4]);
            }

            #[test]
            #[should_panic]
            fn incorrect_json_type() {
                Subject::from_json_unchecked(json!({}));
            }
        }}

        test_mod!{ to_json {
            #[test]
            fn happy_path() {
                let subject: Subject = vec![1, 2, 3, 4];
                let json = subject.to_json();

                assert_eq!(json, json!([1, 2, 3, 4]));
            }
        }}

        test_mod!{ validate_json {
            #[test]
            fn happy_path() {
                assert!(Subject::validate_json(&json!([1])).is_ok());
            }

            #[test]
            fn incorrect_json_type() {
                let result = Subject::validate_json(&json!({}));
                match result {
                    Err(err) => assert_eq!(err, JsonableError::IncompatibleJsonType { got: "object", expected: "array" }),
                    _ => assert!(false)
                };
            }
        }}
    }}
}
