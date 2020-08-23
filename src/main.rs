extern crate rust_map_macro;
use rust_map_macro::mapper::{Mapper, Time};
use std::collections::HashMap;
#[macro_use]
extern crate mapper_derive;

fn main() {
    let test = Test {
        null: None,
        boolean: true,
        int: 1234,
        float: 56.78,
        string: "str".to_string(),
        time: Time {
            seconds: 5678,
            nanos: 1234,
        },
        array: vec!["array1".to_string(), "array2".to_string()],
        map: vec![
            ("key1".to_string(), 1),
            ("key2".to_string(), 2),
            ("key3".to_string(), 3),
        ]
        .into_iter()
        .collect(),
    };

    let test_map = test.to_map();
    println!("map");
    println!("{:?}", test_map);

    let test = Test::from_map(test_map).unwrap();
    println!("struct");
    println!("{:?}", test);
}

#[derive(Debug, Mapper)]
struct Test {
    null: Option<String>,
    boolean: bool,
    int: i64,
    float: f32,
    string: String,
    time: Time,
    array: Vec<String>,
    map: HashMap<String, i64>,
}

#[derive(Debug, Mapper)]
struct Test2 {
    test: Test,
}
