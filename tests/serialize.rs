use std::rc::Rc;

use facet::Facet;
use facet_v8::{Constructors, from_v8, to_v8, to_v8_with_constructors};

mod util;
use util::{check_function, compile_function, run};

#[test]
fn scalar() {
    run(|scope| {
        let uint = to_v8(scope, &123u32).unwrap();
        assert_eq!(
            v8::Local::<v8::Number>::try_from(uint)
                .expect("expected number")
                .value(),
            123.0
        );
        assert_eq!(from_v8::<u32>(scope, uint).unwrap(), 123u32);

        let int = to_v8(scope, &-123i32).unwrap();
        assert_eq!(
            v8::Local::<v8::Number>::try_from(int)
                .expect("expected number")
                .value(),
            -123.0
        );
        assert_eq!(from_v8::<i32>(scope, int).unwrap(), -123i32);

        let float = to_v8(scope, &123.456f64).unwrap();
        assert_eq!(
            v8::Local::<v8::Number>::try_from(float)
                .expect("expected number")
                .value(),
            123.456
        );
        assert_eq!(from_v8::<f64>(scope, float).unwrap(), 123.456f64);

        let boolean = to_v8(scope, &true).unwrap();
        assert!(
            v8::Local::<v8::Boolean>::try_from(boolean)
                .expect("expected boolean")
                .is_true()
        );
        assert!(from_v8::<bool>(scope, boolean).unwrap());
    });
}

#[test]
fn string() {
    run(|scope| {
        let hello = String::from("hello");
        let value = to_v8(scope, &hello).unwrap();
        assert_eq!(
            v8::Local::<v8::String>::try_from(value)
                .expect("expected string")
                .to_rust_string_lossy(scope),
            "hello"
        );
        assert_eq!(from_v8::<String>(scope, value).unwrap(), hello);

        // BUG: https://github.com/facet-rs/facet/issues/794
        // let cow = Cow::Borrowed("hello");
        // let value = to_v8(scope, &cow).unwrap();
        // assert_eq!(
        //     v8::Local::<v8::String>::try_from(value)
        //         .expect("expected string")
        //         .to_rust_string_lossy(scope),
        //     "hello"
        // );
        // assert_eq!(from_v8::<Cow<str>>(scope, value).unwrap(), cow);

        // assert_eq!(
        //     v8::Local::<v8::String>::try_from(to_v8(scope, &&"hello").unwrap())
        //         .expect("expected string")
        //         .to_rust_string_lossy(scope),
        //     "hello"
        // );
        assert_eq!(
            v8::Local::<v8::String>::try_from(to_v8(scope, &String::from("hello")).unwrap())
                .expect("expected string")
                .to_rust_string_lossy(scope),
            "hello"
        );
        // assert_eq!(
        //     v8::Local::<v8::String>::try_from(to_v8(scope, &&String::from("hello")).unwrap())
        //         .expect("expected string")
        //         .to_rust_string_lossy(scope),
        //     "hello"
        // );
        // assert_eq!(
        //     v8::Local::<v8::String>::try_from(to_v8(scope, &&&String::from("hello")).unwrap())
        //         .expect("expected string")
        //         .to_rust_string_lossy(scope),
        //     "hello"
        // );

        let foo = to_v8(scope, &"foo").unwrap();
        check_function(
            scope,
            "check",
            &[foo],
            r#"function check(s) { if (s !== 'foo') { throw new Error('Expected "foo"'); } }"#,
        );
    });
}

#[test]
fn array() {
    run(|scope| {
        let array = to_v8(scope, &[1, 2, 3]).unwrap();
        check_function(
            scope,
            "check",
            &[array],
            r#"
            function check(arr) {
                if (arr.length !== 3 || arr[0] !== 1 || arr[1] !== 2 || arr[2] !== 3) {
                    throw new Error('Expected [1, 2, 3]');
                }
            }"#,
        );
        assert_eq!(from_v8::<Vec<i32>>(scope, array).unwrap(), vec![1, 2, 3]);

        let vec = to_v8(scope, &vec![1, 2, 3]).unwrap();
        check_function(
            scope,
            "check",
            &[vec],
            r#"function check(arr) {
                if (arr.length !== 3 || arr[0] !== 1 || arr[1] !== 2 || arr[2] !== 3) {
                    throw new Error('Expected [1, 2, 3]');
                }
            }"#,
        );

        let boxed = to_v8(scope, &Box::new([1, 2, 3])).unwrap();
        check_function(
            scope,
            "check",
            &[boxed],
            r#"function check(arr) {
                if (arr.length !== 3 || arr[0] !== 1 || arr[1] !== 2 || arr[2] !== 3) {
                    throw new Error('Expected [1, 2, 3]');
                }
            }"#,
        );
    })
}

#[derive(Facet, PartialEq, Debug)]
struct Plain {
    a: i32,
    b: String,
    c: f64,
}

#[test]
fn object() {
    run(|scope| {
        let plain = to_v8(
            scope,
            &Plain {
                a: 42,
                b: "hello".to_string(),
                c: 3.4,
            },
        )
        .unwrap();
        check_function(
            scope,
            "check",
            &[plain],
            r#"function check(obj) {
                if (typeof obj !== 'object' || obj.a !== 42 || obj.b !== 'hello' || obj.c !== 3.4) {
                    throw new Error('Expected { a: 42, b: "hello", c: 3.4 }');
                }
            }"#,
        );
        assert_eq!(
            from_v8::<Plain>(scope, plain).unwrap(),
            Plain {
                a: 42,
                b: "hello".to_string(),
                c: 3.4,
            }
        );

        let constructor = compile_function(
            scope,
            "Foo",
            r#"function Foo() {
                this.x = 10;
                this.y = 20;
            }"#,
        );

        let plain_with_constructor = to_v8_with_constructors(
            scope,
            &Plain {
                a: 42,
                b: "hello".to_string(),
                c: 3.4,
            },
            Constructors::default().with_constructor::<Plain>(constructor),
        )
        .unwrap();
        check_function(
            scope,
            "check",
            &[plain_with_constructor],
            r#"function check(obj) {
                if (typeof obj !== 'object' || obj.a !== 42 || obj.b !== 'hello' || obj.c !== 3.4) {
                    throw new Error('Expected { a: 42, b: "hello", c: 3.4 }');
                }
                if (obj.constructor.name !== 'Foo') {
                    throw new Error('Expected constructor name "Foo"');
                }
                if (obj.x !== 10 || obj.y !== 20) {
                    throw new Error('Expected constructor properties x: 10 and y: 20');
                }
            }"#,
        );
        // This should succeed even though the JS constructor adds fields via the prototype.
        assert_eq!(
            from_v8::<Plain>(scope, plain_with_constructor).unwrap(),
            Plain {
                a: 42,
                b: "hello".to_string(),
                c: 3.4,
            }
        );
    })
}

#[derive(Facet)]
struct PlainRcs {
    a: Rc<Plain>,
    b: Rc<Plain>,
}

#[test]
fn smart_pointers() {
    run(|scope| {
        let plain = Rc::new(Plain {
            a: 42,
            b: "hello".to_string(),
            c: 3.4,
        });
        let plain_rcs = to_v8(
            scope,
            &PlainRcs {
                a: plain.clone(),
                b: plain,
            },
        );
        // Check that `a` and `b` are the same object.
        check_function(
            scope,
            "check",
            &[plain_rcs.unwrap()],
            r#"function check(obj) {
                if (typeof obj !== 'object') {
                    throw new Error('expected object');
                }
                if (obj.a !== obj.b) {
                    throw new Error('expected a and b to be the same object');
                }
                if (obj.a.a !== 42 || obj.a.b !== 'hello' || obj.a.c !== 3.4) {
                    throw new Error('expected a to have properties { a: 42, b: "hello", c: 3.4 }');
                }
            }"#,
        );
    })
}

#[derive(Facet, PartialEq, Debug)]
#[facet(js_enum_repr = "string")]
#[repr(u8)]
enum StringyEnum {
    A = 1,
    B = 2,
    C = 3,
}

#[derive(Facet, PartialEq, Debug)]
#[facet(js_enum_repr = "number")]
#[repr(u8)]
enum NumberEnum {
    A = 1,
    B = 2,
    C = 3,
}

#[test]
fn simple_enums() {
    run(|scope| {
        let a = to_v8(scope, &StringyEnum::A).unwrap();
        let b = to_v8(scope, &StringyEnum::B).unwrap();
        let c = to_v8(scope, &StringyEnum::C).unwrap();
        check_function(
            scope,
            "check",
            &[a, b, c],
            r#"function check(a, b, c) {
                if (a !== 'A' || b !== 'B' || c !== 'C') {
                    throw new Error(`Expected "A", "B", "C" (got ${a}, ${b}, ${c})`);
                }
            }"#,
        );
        assert_eq!(from_v8::<StringyEnum>(scope, a).unwrap(), StringyEnum::A);
        assert_eq!(from_v8::<StringyEnum>(scope, b).unwrap(), StringyEnum::B);
        assert_eq!(from_v8::<StringyEnum>(scope, c).unwrap(), StringyEnum::C);

        let a = to_v8(scope, &NumberEnum::A).unwrap();
        let b = to_v8(scope, &NumberEnum::B).unwrap();
        let c = to_v8(scope, &NumberEnum::C).unwrap();
        check_function(
            scope,
            "check",
            &[a, b, c],
            r#"function check(a, b, c) {
                if (a !== 1 || b !== 2 || c !== 3) {
                    throw new Error(`Expected "A", "B", "C" (got ${a}, ${b}, ${c})`);
                }
            }"#,
        );
        assert_eq!(from_v8::<NumberEnum>(scope, a).unwrap(), NumberEnum::A);
        assert_eq!(from_v8::<NumberEnum>(scope, b).unwrap(), NumberEnum::B);
        assert_eq!(from_v8::<NumberEnum>(scope, c).unwrap(), NumberEnum::C);
    })
}

#[derive(Facet, PartialEq, Debug)]
#[repr(u8)]
enum ComplexEnum {
    Unit,
    Tuple(i32, String),
    Struct { a: i32, b: String },
}

#[test]
fn complex_enum() {
    run(|scope| {
        let unit = to_v8(scope, &ComplexEnum::Unit).unwrap();
        let tuple = to_v8(scope, &ComplexEnum::Tuple(42, "hello".to_string())).unwrap();
        let struct_ = to_v8(
            scope,
            &ComplexEnum::Struct {
                a: 42,
                b: "hello".to_string(),
            },
        )
        .unwrap();

        check_function(
            scope,
            "check",
            &[unit, tuple, struct_],
            r#"function check(unit, tuple, struct) {
                if (unit.type !== 'Unit') {
                    throw new Error(`Expected "Unit", got ${JSON.stringify(unit)}`);
                }
                if (tuple.type !== 'Tuple' || tuple[0] !== 42 || tuple[1] !== 'hello') {
                    throw new Error(`Expected Tuple(42, "hello"), got ${JSON.stringify(tuple)}`);
                }
                if (struct.type !== 'Struct' || struct.a !== 42 || struct.b !== 'hello') {
                    throw new Error(`Expected Struct { a: 42, b: "hello" }, got ${JSON.stringify(struct)}`);
                }
            }"#,
        );

        assert_eq!(
            from_v8::<ComplexEnum>(scope, unit).unwrap(),
            ComplexEnum::Unit
        );
        assert_eq!(
            from_v8::<ComplexEnum>(scope, tuple).unwrap(),
            ComplexEnum::Tuple(42, "hello".to_string())
        );
        assert_eq!(
            from_v8::<ComplexEnum>(scope, struct_).unwrap(),
            ComplexEnum::Struct {
                a: 42,
                b: "hello".to_string()
            }
        );
    })
}

#[derive(Facet, PartialEq, Debug)]
struct TypedArray<T> {
    #[facet(typed_array)]
    data: Vec<T>,
}

#[test]
fn typed_arrays_u8() {
    run(|scope| {
        let array = TypedArray {
            data: vec![1u8, 2, 3],
        };
        let v8_array = to_v8(scope, &array).unwrap();
        check_function(
            scope,
            "check",
            &[v8_array],
            r#"function check(array) {
                if (!(array.data instanceof Uint8Array)) {
                    throw new Error(`Expected Uint8Array, got ${array}`);
                }
                if (array.data.length !== 3 || array.data[0] !== 1 || array.data[1] !== 2 || array.data[2] !== 3) {
                    throw new Error(`Expected [1, 2, 3], got ${array}`);
                }
            }"#,
        );
        assert_eq!(from_v8::<TypedArray<u8>>(scope, v8_array).unwrap(), array);
    })
}

#[test]
fn typed_arrays_i32() {
    run(|scope| {
        let array = TypedArray {
            data: vec![1i32, 2, 3],
        };
        let v8_array = to_v8(scope, &array).unwrap();
        check_function(
            scope,
            "check",
            &[v8_array],
            r#"function check(array) {
                if (!(array.data instanceof Int32Array)) {
                    throw new Error(`Expected Int32Array, got ${array}`);
                }
                if (array.data.length !== 3 || array.data[0] !== 1 || array.data[1] !== 2 || array.data[2] !== 3) {
                    throw new Error(`Expected [1, 2, 3], got ${array}`);
                }
            }"#,
        );
        assert_eq!(from_v8::<TypedArray<i32>>(scope, v8_array).unwrap(), array);
    })
}

#[test]
fn typed_arrays_f64() {
    run(|scope| {
        let array = TypedArray {
            data: vec![1.0f64, 2.0, 3.0],
        };
        let v8_array = to_v8(scope, &array).unwrap();
        check_function(
            scope,
            "check",
            &[v8_array],
            r#"function check(array) {
                if (!(array.data instanceof Float64Array)) {
                    throw new Error(`Expected Float64Array, got ${array}`);
                }
                if (array.data.length !== 3 || array.data[0] !== 1.0 || array.data[1] !== 2.0 || array.data[2] !== 3.0) {
                    throw new Error(`Expected [1.0, 2.0, 3.0], got ${array}`);
                }
            }"#,
        );
        assert_eq!(from_v8::<TypedArray<f64>>(scope, v8_array).unwrap(), array);
    })
}
