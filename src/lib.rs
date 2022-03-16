//! Jsonable provides a easy way to convert from and to [serde_json::Value](https://docs.serde.rs/serde_json/value/enum.Value.html)
//! for your types while in memory, allowing you to modify the json before
//! converting to your type.
//! 
//! # Example
//! ```
//! use json_patch::patch;
//! use serde_json::*;
//! use jsonable::*;
//! 
//! #[derive(Debug,Jsonable)]
//! struct Person {
//!     pub first_name: String,
//!     pub last_name: Option<String>
//! }
//! 
//! let mut doc = json!({ "first_name": "Andrew" });
//! 
//! let p = from_str(r#"[
//!         { "op": "test", "path": "/first_name", "value": "Andrew" },
//!         { "op": "add", "path": "/last_name", "value": "Marx" }
//!     ]"#).unwrap();
//! 
//! patch(&mut doc, &p).unwrap();
//! 
//! let person = Person::from_json(doc).unwrap();
//! 
//! assert_eq!(person.last_name, Some("Marx".into()))
//! ```
//!
//! # Roadmap
//! - [X] Implement derive for Named Structs
//! - [ ] Implement derive for Tuple Structs
//! - [ ] Implement derive for Enums
//! - [ ] Add helper attributes to allow mapping json keys to fields/values
pub use jsonable_macros::*;

pub use jsonable_types::*;


#[cfg(test)]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/named_structs/happy_path.rs");
    t.compile_fail("tests/ui/enum/unimplemented.rs");
    t.compile_fail("tests/ui/tuple_structs/unimplemented.rs");
}

