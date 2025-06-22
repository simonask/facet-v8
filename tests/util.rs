static INIT_V8: std::sync::Once = std::sync::Once::new();

pub fn run(f: impl FnOnce(&mut v8::HandleScope<v8::Context>)) {
    INIT_V8.call_once(|| {
        v8::V8::set_flags_from_string("--single_threaded");
        v8::V8::initialize_platform(v8::new_single_threaded_default_platform(false).make_shared());
        v8::V8::initialize();
    });

    let mut isolate = v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(&mut isolate);
    let context = v8::Context::new(scope, v8::ContextOptions::default());
    let scope = &mut v8::ContextScope::new(scope, context);
    f(scope);
}

#[allow(dead_code)]
pub fn compile_function<'s>(
    scope: &mut v8::HandleScope<'s>,
    check_function_name: &str,
    script: &str,
) -> v8::Local<'s, v8::Function> {
    let mut scope = v8::TryCatch::new(scope);
    let source = v8::String::new(&mut scope, script).expect("script too large");
    let function_name =
        v8::String::new(&mut scope, check_function_name).expect("function name too large");
    let Some(script) = v8::Script::compile(&mut scope, source, None) else {
        fail_on_exception(&mut scope);
    };
    let Some(_) = script.run(&mut scope) else {
        fail_on_exception(&mut scope);
    };
    let global = scope.get_current_context().global(&mut scope);
    let Some(function) = global.get(&mut scope, function_name.into()) else {
        fail_on_exception(&mut scope);
    };
    let Ok(func) = v8::Local::<v8::Function>::try_from(function) else {
        panic!("`{check_function_name}` is not a function");
    };
    func
}

#[allow(dead_code)]
pub fn check_function<'s>(
    scope: &mut v8::HandleScope<'s>,
    check_function_name: &str,
    arguments: &[v8::Local<'s, v8::Value>],
    script: &str,
) {
    let mut scope = v8::TryCatch::new(scope);
    let func = compile_function(&mut scope, check_function_name, script);
    let global = scope.get_current_context().global(&mut scope);
    let Some(_) = func.call(&mut scope, global.into(), arguments) else {
        fail_on_exception(&mut scope);
    };
}

#[allow(dead_code)]
pub fn fail_on_exception(scope: &mut v8::TryCatch<v8::HandleScope>) -> ! {
    let exception = scope.exception().unwrap();
    let message = exception.to_string(scope).expect("no exception message");
    panic!("V8 exception: {}", message.to_rust_string_lossy(scope));
}
