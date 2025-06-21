use std::rc::Rc;

use facet::Facet;
use facet_v8::{Constructors, to_v8, to_v8_with_constructors};
use serial_test::serial;

mod util;
use util::{check_function, compile_function, run};

#[test]
#[serial]
fn serialize_scalar() {
    run(|scope| {
        let uint = to_v8(scope, &123u32).unwrap();
        assert_eq!(
            v8::Local::<v8::Number>::try_from(uint)
                .expect("expected number")
                .value(),
            123.0
        );

        let int = to_v8(scope, &-123i32).unwrap();
        assert_eq!(
            v8::Local::<v8::Number>::try_from(int)
                .expect("expected number")
                .value(),
            -123.0
        );

        let float = to_v8(scope, &123.456f64).unwrap();
        assert_eq!(
            v8::Local::<v8::Number>::try_from(float)
                .expect("expected number")
                .value(),
            123.456
        );

        let boolean = to_v8(scope, &true).unwrap();
        assert!(
            v8::Local::<v8::Boolean>::try_from(boolean)
                .expect("expected boolean")
                .is_true()
        );
    });
}

#[test]
#[serial]
fn serialize_string() {
    run(|scope| {
        assert_eq!(
            v8::Local::<v8::String>::try_from(to_v8(scope, &"hello").unwrap())
                .expect("expected string")
                .to_rust_string_lossy(scope),
            "hello"
        );
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
#[serial]
fn serialize_array() {
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

#[derive(Facet)]
struct Plain {
    a: i32,
    b: String,
    c: f64,
}

#[test]
#[serial]
fn serialize_object() {
    run(|scope| {
        let plain = to_v8(
            scope,
            &Plain {
                a: 42,
                b: "hello".to_string(),
                c: 3.4,
            },
        );
        check_function(
            scope,
            "check",
            &[plain.unwrap()],
            r#"function check(obj) {
                if (typeof obj !== 'object' || obj.a !== 42 || obj.b !== 'hello' || obj.c !== 3.4) {
                    throw new Error('Expected { a: 42, b: "hello", c: 3.4 }');
                }
            }"#,
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
    })
}

#[derive(Facet)]
struct PlainRcs {
    a: Rc<Plain>,
    b: Rc<Plain>,
}

#[test]
#[serial]
fn serialize_smart_pointers() {
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

#[derive(Facet)]
#[facet(js_enum_repr = "string")]
#[repr(u8)]
enum StringyEnum {
    A = 1,
    B = 2,
    C = 3,
}

#[derive(Facet)]
#[facet(js_enum_repr = "number")]
#[repr(u8)]
enum NumberEnum {
    A = 1,
    B = 2,
    C = 3,
}

#[test]
#[serial]
fn serialize_simple_enums() {
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
    })
}

#[derive(Facet)]
#[repr(u8)]
enum ComplexEnum {
    Unit,
    Tuple(i32, String),
    Struct { a: i32, b: String },
}

#[test]
#[serial]
fn serialize_complex_enum() {
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
    })
}
