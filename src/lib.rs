//! Jsonable provides a easy way to convert from and to [serde_json::Value](https://docs.serde.rs/serde_json/value/enum.Value.html)
//! for your types while in memory, allowing you to modify the json before
//! converting to your type.
//! 
//! # Example
//! ```ignore
//! #[macro_use]
//! extern crate serde_json;
//! extern crate json_patch;
//! extern crate jsonable;
//! 
//! use json_patch::patch;
//! use serde_json::from_str;
//! use jsonable::Jsonable;
//! 
//! struct Person {
//!     pub first_name: String
//!     pub last_name: Option<String>
//! }
//! 
//! let mut doc = json!({ "first_name": "Andrew" });
//! 
//! let p = from_str(r#"[
//!   { "op": "test", "path": "/0/name", "value": "Andrew" },
//!   { "op": "add", "path": "/0/last_name", "value": "Marx" }
//! ]"#).unwrap();
//! 
//! patch(&mut doc, &p).unwrap();
//! 
//! let person: Person = Person::from_json(doc);
//! assert_eq!(person.last_name, Some("Marx".into()))
//! ```
//! 
pub use jsonable_macros::*;

pub use jsonable_types::*;

use crate as jsonable;

#[derive(Jsonable)]
pub struct Test {
    pub value: u8,
    pub complex: Vec<u8>
}