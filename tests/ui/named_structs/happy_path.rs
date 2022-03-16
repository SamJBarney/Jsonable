use jsonable::*;

use std::collections::{HashSet, HashMap};

#[derive(Jsonable)]
struct Simple {
    pub something: u8,
    pub value: String
}

#[derive(Jsonable)]
struct Complex {
    pub vec: Vec<HashSet<String>>,
    pub map: HashMap<String, Vec<String>>
}

fn main() {
    let _subject = Simple { something: 8, value: "string".into() };
    let _complex = Complex { vec: Vec::new(), map: HashMap::new() };
}