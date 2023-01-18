use jsonable::*;

#[derive(Jsonable)]
enum Simple {
    Value,
    Value2
}

#[derive(Jsonable)]
enum ComplexUnnamed {
    Single(u32),
    OptionalSingle(Option<u32>),
    Multiple(u32, u16),
    EvenMoreMultiple(Simple, u32, i8, Option<String>),
    NamedSingle{ gregory: usize },
    NamedMultiple{ gregistan: isize, count: u16, marker: Option<Simple> }
}

fn main()  {}