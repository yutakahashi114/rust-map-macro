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
        array: vec!["array1".to_string(), "array2".to_string()],
        map: vec![("seconds".to_string(), 1234), ("nanos".to_string(), 5678)]
            .into_iter()
            .collect(),
        time: Time {
            seconds: 1234,
            nanos: 5678,
        },
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
    float: f64,
    string: String,
    array: Vec<String>,
    map: HashMap<String, i64>,
    time: Time,
}
