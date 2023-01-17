use serde::{Serialize, Deserialize};

#[repr(u16)]
#[derive(Debug, Serialize, Deserialize)]
enum TestEnum {
    Value,
    Value2 = 1,
    Anonymouse(f64, String),
    Named{field1: f64},
    NamedMultiple{field1:f64, field2: f64},
    USize(usize),
    Float(f32),
    Double(f64),
    Box(Box<TestEnum>)
}

#[derive(Debug, Serialize, Deserialize)]
struct TestStruct {
    pub value: TestEnum
}

fn main() {
    let mut result = serde_json::ser::to_string(&TestStruct { value: TestEnum::USize(12) }).unwrap();
    println!("{}", result);
    result = serde_json::ser::to_string(&TestStruct { value: TestEnum::Float(12.0) }).unwrap();
    println!("{}", result);
    result = serde_json::ser::to_string(&TestStruct { value: TestEnum::Double(12.0) }).unwrap();
    println!("{}", result);
    result = serde_json::ser::to_string(&TestStruct { value: TestEnum::Box(TestEnum::Float(12.0).into()) }).unwrap();
    println!("{}", result);
    result = serde_json::ser::to_string(&TestStruct { value: TestEnum::Value }).unwrap();
    println!("{}", result);
    result = serde_json::ser::to_string(&TestStruct { value: TestEnum::Value2 }).unwrap();
    println!("{}", result);
    result = serde_json::ser::to_string(&TestStruct { value: TestEnum::Anonymouse(12.0, "Hello".into()) }).unwrap();
    println!("{}", result);
    result = serde_json::ser::to_string(&TestStruct { value: TestEnum::Named{field1: 12.0} }).unwrap();
    println!("{}", result);
    result = serde_json::ser::to_string(&TestStruct { value: TestEnum::NamedMultiple { field1: 12.0, field2: 13.9 } }).unwrap();
    println!("{}", result);
}
