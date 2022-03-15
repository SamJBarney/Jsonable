use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use serde_json::{self, Value, Map};


#[derive(Debug)]
pub enum JsonableError {
    IncompatibleJsonType {got: &'static str, expected: &'static str },
    IncompatibleEntryForType(&'static str),
    InnerErrorForType{ty: &'static str, error: Box<JsonableError>}
}

pub type JsonResult<T> = Result<T, JsonableError>;

pub trait Jsonable: Sized {
    fn from_json(json: Value) -> JsonResult<Self> {
        match Self::verify_json(&json) {
            Ok(_) => Ok(Self::from_json_unchecked(json)),
            Err(err) => Err(err)
        }
    }
    fn from_json_unchecked(json: Value) -> Self;
    fn to_json(&self) -> Value;
    fn verify_json(json: &Value) -> JsonResult<()>;
}

impl<T:Jsonable> Jsonable for Vec<T> {
    fn from_json_unchecked(mut json: Value) -> Self {
        json.as_array_mut().unwrap().drain(..).map(|value| { T::from_json_unchecked(value) }).collect::<Self>()
    }

    fn to_json(&self) -> Value {
        Value::Array(self.into_iter().map(|entry| { entry.to_json() }).collect())
    }

    fn verify_json(json: &Value) -> Result<(), JsonableError> {
        match json {
            Value::Array(vec) => {
                if vec.into_iter().all(|entry| {
                    match T::verify_json(&entry) {
                        Ok(_) => true,
                        Err(_) => false
                    }
                }) {
                    Ok(())
                } else {
                    Err(JsonableError::IncompatibleEntryForType(std::any::type_name::<T>()))
                }
            },
            Value::Bool(_) => Err(JsonableError::IncompatibleJsonType { got: "bool", expected: "array" }),
            Value::Null => Err(JsonableError::IncompatibleJsonType { got: "null", expected: "array" }),
            Value::Number(_) => Err(JsonableError::IncompatibleJsonType { got: "number", expected: "array" }),
            Value::Object(_) => Err(JsonableError::IncompatibleJsonType { got: "object", expected: "array" }),
            Value::String(_) => Err(JsonableError::IncompatibleJsonType { got: "string", expected: "array" })
        }
    }
}

impl <I, T> Jsonable for HashMap<I, T>
    where I: From<String> + Into<String> + Hash + Eq + Clone,
    T: Jsonable,
    String: From<I> {
    fn from_json_unchecked(json: Value) -> Self {
        let obj = json.as_object().unwrap();
        let mut map = HashMap::with_capacity(obj.keys().len());
        for (key, value) in obj.into_iter() {
            map.insert(I::from(key.to_owned()), T::from_json_unchecked(value.to_owned()));
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

    fn verify_json(json: &Value) -> Result<(), JsonableError> {
        match json {
            Value::Object(map) => {
                if map.values().all(|value| {
                    match T::verify_json(value) {
                        Ok(()) => true,
                        _ => false
                    }
                }) {
                    Ok(())
                } else {
                    Err(JsonableError::IncompatibleEntryForType(std::any::type_name::<T>()))
                }
            }
            Value::Array(_) => Err(JsonableError::IncompatibleJsonType { got: "array", expected: "object" }),
            Value::Bool(_) => Err(JsonableError::IncompatibleJsonType { got: "bool", expected: "object" }),
            Value::Null => Err(JsonableError::IncompatibleJsonType { got: "null", expected: "object" }),
            Value::Number(_) => Err(JsonableError::IncompatibleJsonType { got: "number", expected: "object" }),
            Value::String(_) => Err(JsonableError::IncompatibleJsonType { got: "string", expected: "object" })
        }
    }
}

impl<T> Jsonable for HashSet<T> where T: Jsonable + Eq + Hash {
    fn from_json_unchecked(mut json: Value) -> Self {
        let vec = json.as_array_mut().unwrap();
        let mut set = HashSet::with_capacity(vec.len());
        for value in  vec.drain(..) {
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

    fn verify_json(json: &Value) -> Result<(), JsonableError> {
        Vec::<T>::verify_json(json)
    }
}

impl<T> Jsonable for Option<T> where T: Jsonable {
    fn from_json_unchecked(json: Value) -> Self {
        match json {
            Value::Null => None,
            _ => Some (T::from_json_unchecked(json))
        }
        
    }

    fn to_json(&self) -> Value {
        if let Some(value) = self {
            value.to_json()
        } else {
            Value::Null
        }
    }

    fn verify_json(json: &Value) -> Result<(), JsonableError> {
        match json {
            Value::Null => Ok(()),
            _ => T::verify_json(json)
        }
    }
}

impl Jsonable for String {
    fn from_json_unchecked(json: Value) -> Self {
        json.to_string()
    }

    fn to_json(&self) -> Value {
        Value::String(self.clone())
    }

    fn verify_json(json: &Value) -> Result<(), JsonableError> {
        match json {
            Value::String(_) => Ok(()),
            Value::Null => Err(JsonableError::IncompatibleJsonType { got: "null", expected: "string" }),
            Value::Bool(_) => Err(JsonableError::IncompatibleJsonType { got: "bool", expected: "string" }),
            Value::Number(_) => Err(JsonableError::IncompatibleJsonType { got: "number", expected: "string" }),
            Value::Array(_) => Err(JsonableError::IncompatibleJsonType { got: "array", expected: "string" }),
            Value::Object(_) => Err(JsonableError::IncompatibleJsonType { got: "object", expected: "string" }),
        }
    }
}

macro_rules! number_impl {
    ($ty: ty, $method: ident) => {
        impl Jsonable for $ty {
            fn from_json_unchecked(json: Value) -> Self {
                json.$method().unwrap() as $ty
            }
        
            fn to_json(&self) -> Value {
                Value::from(*self)
            }
        
            fn verify_json(json: &Value) -> Result<(), JsonableError> {
                match json {
                    Value::Number(_) => Ok(()),
                    Value::Array(_) => Err(JsonableError::IncompatibleJsonType { got: "array", expected: "number" }),
                    Value::Bool(_) => Err(JsonableError::IncompatibleJsonType { got: "bool", expected: "number" }),
                    Value::Null => Err(JsonableError::IncompatibleJsonType { got: "null", expected: "number" }),
                    Value::Object(_) => Err(JsonableError::IncompatibleJsonType { got: "object", expected: "number" }),
                    Value::String(_) => Err(JsonableError::IncompatibleJsonType { got: "string", expected: "number" })
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