#![allow(clippy::approx_constant)]

use divan::{Bencher, black_box};
use facet::Facet;
use serde::{Deserialize, Serialize};
// use std::collections::HashMap;

#[path = "../tests/util.rs"]
mod util;
use util::run;

#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested0 {
    id: u64,
    name: String,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested1 {
    id: u64,
    name: String,
    child: Nested0,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested2 {
    id: u64,
    name: String,
    child: Nested1,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested3 {
    id: u64,
    name: String,
    child: Nested2,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested4 {
    id: u64,
    name: String,
    child: Nested3,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested5 {
    id: u64,
    name: String,
    child: Nested4,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested6 {
    id: u64,
    name: String,
    child: Nested5,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested7 {
    id: u64,
    name: String,
    child: Nested6,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested8 {
    id: u64,
    name: String,
    child: Nested7,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested9 {
    id: u64,
    name: String,
    child: Nested8,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested10 {
    id: u64,
    name: String,
    child: Nested9,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested11 {
    id: u64,
    name: String,
    child: Nested10,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested12 {
    id: u64,
    name: String,
    child: Nested11,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested13 {
    id: u64,
    name: String,
    child: Nested12,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested14 {
    id: u64,
    name: String,
    child: Nested13,
}
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
pub struct Nested15 {
    id: u64,
    name: String,
    child: Nested14,
}

// Wide Structure
#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
struct Wide {
    field01: String,
    field02: u64,
    field03: i32,
    field04: f64,
    field05: bool,
    field06: Option<String>,
    field07: Vec<u32>,
    field08: String,
    field09: u64,
    field10: i32,
    field11: f64,
    field12: bool,
    field13: Option<String>,
    field14: Vec<u32>,
    field15: String,
    field16: u64,
    field17: i32,
    field18: f64,
    field19: bool,
    field20: Option<String>,
    field21: Vec<u32>,
    field22: String,
    field23: u64,
    field24: i32,
    field25: f64,
    field26: bool,
    field27: Option<String>,
    field28: Vec<u32>,
    // field29: HashMap<String, i32>,
    field30: Nested0,
}

fn create_wide() -> Wide {
    // let mut map = HashMap::new();
    // map.insert("a".to_string(), 1);
    // map.insert("b".to_string(), 2);

    Wide {
        field01: "value 01".to_string(),
        field02: 1234567890123456789,
        field03: -123456789,
        field04: 3.141592653589793,
        field05: true,
        field06: Some("optional value 06".to_string()),
        field07: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
        field08: "value 08".to_string(),
        field09: 9876543210987654321,
        field10: 987654321,
        field11: 2.718281828459045,
        field12: false,
        field13: None,
        field14: vec![0, 9, 8, 7, 6, 5, 4, 3, 2, 1],
        field15: "value 15".to_string(),
        field16: 1111111111111111111,
        field17: -111111111,
        field18: 1.618033988749895,
        field19: true,
        field20: Some("optional value 20".to_string()),
        field21: vec![10, 20, 30],
        field22: "value 22".to_string(),
        field23: 2222222222222222222,
        field24: -222222222,
        field25: 0.5772156649015329,
        field26: false,
        field27: None,
        field28: vec![],
        // field29: map,
        field30: Nested0 {
            id: 0,
            name: "Base Nested".to_string(),
        },
    }
}

// Helper function to create nested test data
fn create_nested_data() -> Vec<Nested15> {
    let data = Nested15 {
        id: 15,
        name: "Level 15".to_string(),
        child: Nested14 {
            id: 14,
            name: "Level 14".to_string(),
            child: Nested13 {
                id: 13,
                name: "Level 13".to_string(),
                child: Nested12 {
                    id: 12,
                    name: "Level 12".to_string(),
                    child: Nested11 {
                        id: 11,
                        name: "Level 11".to_string(),
                        child: Nested10 {
                            id: 10,
                            name: "Level 10".to_string(),
                            child: Nested9 {
                                id: 9,
                                name: "Level 9".to_string(),
                                child: Nested8 {
                                    id: 8,
                                    name: "Level 8".to_string(),
                                    child: Nested7 {
                                        id: 7,
                                        name: "Level 7".to_string(),
                                        child: Nested6 {
                                            id: 6,
                                            name: "Level 6".to_string(),
                                            child: Nested5 {
                                                id: 5,
                                                name: "Level 5".to_string(),
                                                child: Nested4 {
                                                    id: 4,
                                                    name: "Level 4".to_string(),
                                                    child: Nested3 {
                                                        id: 3,
                                                        name: "Level 3".to_string(),
                                                        child: Nested2 {
                                                            id: 2,
                                                            name: "Level 2".to_string(),
                                                            child: Nested1 {
                                                                id: 1,
                                                                name: "Level 1".to_string(),
                                                                child: Nested0 {
                                                                    id: 0,
                                                                    name: "Level 0".to_string(),
                                                                },
                                                            },
                                                        },
                                                    },
                                                },
                                            },
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        },
    };
    vec![data.clone(); 100]
}
//
// Nested benchmark functions

#[divan::bench(name = "Serialize - Nested (depth=15) - facet_v8")]
fn bench_nested_facet_v8_serialize(bencher: Bencher) {
    let data = create_nested_data();

    run(|scope| {
        bencher.bench_local(|| black_box(facet_v8::to_v8(scope, black_box(&data))));
    })
}

#[divan::bench(name = "Serialize - Nested (depth=15) - serde_v8")]
fn bench_nested_serde_serialize(bencher: Bencher) {
    let data = create_nested_data();

    run(|scope| {
        bencher.bench_local(|| black_box(serde_v8::to_v8(scope, black_box(&data))));
    })
}

#[divan::bench(name = "Deserialize - Nested (depth=15) - facet_v8")]
fn bench_nested_facet_v8_deserialize(bencher: Bencher) {
    let data = create_nested_data();
    run(|scope| {
        let object =
            serde_v8::to_v8(scope, &data).expect("Failed to create nested object for depth 15");

        bencher.bench_local(|| {
            let res: Vec<Nested15> =
                black_box(facet_v8::from_v8(scope, black_box(object))).unwrap();
            black_box(res)
        });
    })
}

#[divan::bench(name = "Deserialize - Nested (depth=15) - serde_v8")]
fn bench_nested_serde_deserialize(bencher: Bencher) {
    let data = create_nested_data();
    run(|scope| {
        let value =
            serde_v8::to_v8(scope, &data).expect("Failed to create nested object for depth 15");

        bencher.bench_local(|| {
            let res: Vec<Nested15> = black_box(serde_v8::from_v8(scope, black_box(value))).unwrap();
            black_box(res)
        });
    });
}

// Wide benchmark functions

#[divan::bench(name = "Serialize - Wide - facet_v8")]
fn bench_wide_facet_v8_serialize(bencher: Bencher) {
    let data = create_wide();
    run(|scope| {
        bencher.bench_local(|| black_box(facet_v8::to_v8(scope, black_box(&data))));
    });
}

#[divan::bench(name = "Serialize - Wide - serde_v8")]
fn bench_wide_serde_serialize(bencher: Bencher) {
    let data = create_wide();

    run(|scope| {
        bencher.bench_local(|| black_box(serde_v8::to_v8(scope, black_box(&data))));
    });
}

#[divan::bench(name = "Deserialize - Wide - facet_v8")]
fn bench_wide_facet_v8_deserialize(bencher: Bencher) {
    let data = create_wide();
    run(|scope| {
        let value = serde_v8::to_v8(scope, &data).expect("Failed to create wide object");

        bencher.bench_local(|| {
            let res: Wide = black_box(facet_v8::from_v8(scope, black_box(value))).unwrap();
            black_box(res)
        });
    });
}

#[divan::bench(name = "Deserialize - Wide - serde_v8")]
fn bench_wide_serde_deserialize(bencher: Bencher) {
    let data = create_wide();
    run(|scope| {
        let value = serde_v8::to_v8(scope, &data).expect("Failed to create wide object");

        bencher.bench_local(|| {
            let res: Wide = black_box(serde_v8::from_v8(scope, black_box(value))).unwrap();
            black_box(res)
        });
    });
}

// Long string benchmark

#[derive(Debug, PartialEq, Clone, Facet, Serialize, Deserialize)]
struct LongString {
    long_string: String,
}

fn create_long_string(size: usize) -> LongString {
    // Create a string of specified size
    let long_str = "a".repeat(size);
    LongString {
        long_string: long_str,
    }
}

#[divan::bench(name = "Serialize - Long String (10KB) - facet_v8")]
fn bench_long_string_facet_v8_serialize(bencher: Bencher) {
    let data = create_long_string(10_000); // 10KB string

    run(|scope| {
        bencher.bench_local(|| black_box(facet_v8::to_v8(scope, black_box(&data))));
    });
}

#[divan::bench(name = "Serialize - Long String (10KB) - serde_v8")]
fn bench_long_string_serde_serialize(bencher: Bencher) {
    let data = create_long_string(10_000); // 10KB string

    run(|scope| {
        bencher.bench_local(|| black_box(serde_v8::to_v8(scope, black_box(&data))));
    });
}

#[divan::bench(name = "Deserialize - Long String (10KB) - facet_v8")]
fn bench_long_string_facet_v8_deserialize(bencher: Bencher) {
    let data = create_long_string(10_000); // 10KB string
    run(|scope| {
        let value = serde_v8::to_v8(scope, &data).expect("Failed to create long string");

        bencher.bench_local(|| {
            let res: LongString = black_box(facet_v8::from_v8(scope, black_box(value))).unwrap();
            black_box(res)
        });
    });
}

#[divan::bench(name = "Deserialize - Long String (10KB) - serde_v8")]
fn bench_long_string_serde_deserialize(bencher: Bencher) {
    let data = create_long_string(10_000); // 10KB string
    run(|scope| {
        let value = serde_v8::to_v8(scope, &data).expect("Failed to create long string");

        bencher.bench_local(|| {
            let res: LongString = black_box(serde_v8::from_v8(scope, black_box(value))).unwrap();
            black_box(res)
        });
    });
}

// Add a larger string test for more SIMD-oriented workloads

#[divan::bench(name = "Serialize - Long String (100KB) - facet_v8")]
fn bench_long_string_100k_facet_v8_serialize(bencher: Bencher) {
    let data = create_long_string(100_000); // 100KB string

    run(|scope| {
        bencher.bench_local(|| black_box(facet_v8::to_v8(scope, black_box(&data))));
    });
}

#[divan::bench(name = "Serialize - Long String (100KB) - serde_v8")]
fn bench_long_string_100k_serde_serialize(bencher: Bencher) {
    let data = create_long_string(100_000); // 100KB string

    run(|scope| {
        bencher.bench_local(|| black_box(serde_v8::to_v8(scope, black_box(&data))));
    });
}

#[divan::bench(name = "Deserialize - Long String (100KB) - facet_v8")]
fn bench_long_string_100k_facet_v8_deserialize(bencher: Bencher) {
    let data = create_long_string(100_000); // 100KB string
    run(|scope| {
        let value = serde_v8::to_v8(scope, &data).expect("Failed to create long string");

        bencher.bench_local(|| {
            let res: LongString = black_box(facet_v8::from_v8(scope, black_box(value))).unwrap();
            black_box(res)
        });
    });
}

#[divan::bench(name = "Deserialize - Long String (100KB) - serde_v8")]
fn bench_long_string_100k_serde_deserialize(bencher: Bencher) {
    let data = create_long_string(100_000); // 100KB string
    run(|scope| {
        let value = serde_v8::to_v8(scope, &data).expect("Failed to create long string");

        bencher.bench_local(|| {
            let res: LongString = black_box(serde_v8::from_v8(scope, black_box(value))).unwrap();
            black_box(res)
        });
    });
}

fn main() {
    divan::main();
}
