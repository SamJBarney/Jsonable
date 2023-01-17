use jsonable::*;

#[derive(Jsonable)]
enum Simple {
    Value
}

#[derive(Jsonable)]
enum ComplexUnnamed {
    Single(u32),
    Multiple(u32, u16),
    EvenMoreMultiple(Simple, u32, i8, String)
}

fn main()  {}