use std::sync::{Arc, Mutex};
use wasmer::{imports, wat2wasm, Function, Instance, Module, Store, WasmerEnv};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_jit::JIT;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_bytes = wat2wasm(
        br#"
(module
  ;; next two lines are added with walrus
  ;;(func $malloc_counter (import "env" "malloc_counter"))
  ;;(func $free_counter (import "env" "free_counter"))

  (func $malloc (param i32) (result i32)
    ;; next line is added with walrus
    ;;(call $malloc_counter)
    ;; some work here
    i32.const 0
  )

  (func $free (param i32)
    ;; next line is added with walrus
    ;;(call $free_counter)
    ;; some work here
  )

  (func (export "main") (result i32)
    (call $malloc (i32.const 0))
    (call $free (i32.const 0))
    i32.const 42
    return))
"#,
    )?;

    // ---------------- walrus transformation
    let config = walrus::ModuleConfig::new();
    let mut walrus_module = config.parse(&wasm_bytes)?;

    let void_type = walrus_module.types.add(&[], &[]);
    // add malloc import
    let (malloc_counter_id, _) = walrus_module.add_import_func("env", "malloc_counter", void_type);

    // add free import
    let (free_counter_id, _) = walrus_module.add_import_func("env", "free_counter", void_type);

    // add extra line to guest malloc
    let malloc_id = walrus_module.funcs.by_name("malloc").unwrap();
    walrus_module
        .funcs
        .iter_local_mut()
        .filter(|(id, _)| id == &malloc_id)
        .next()
        .unwrap()
        .1
        .builder_mut()
        .func_body()
        .call_at(0, malloc_counter_id);
    // add extra line to guest free
    let free_id = walrus_module.funcs.by_name("free").unwrap();
    walrus_module
        .funcs
        .iter_local_mut()
        .filter(|(id, _)| id == &free_id)
        .next()
        .unwrap()
        .1
        .builder_mut()
        .func_body()
        .call_at(0, free_counter_id);

    let transformed_wasm_bytes = walrus_module.emit_wasm();
    // ---------------- walrus transformation

    let store = Store::new(&JIT::new(Cranelift::default()).engine());

    let module = Module::new(&store, transformed_wasm_bytes)?;

    #[derive(WasmerEnv, Clone)]
    struct Env {
        malloc_counter: Arc<Mutex<i32>>,
        free_counter: Arc<Mutex<i32>>,
    }

    let shared_env: Env = Env {
        malloc_counter: Arc::new(Mutex::new(0)),
        free_counter: Arc::new(Mutex::new(0)),
    };

    fn malloc_counter(env: &Env) -> () {
        let mut counter_ref = env.malloc_counter.lock().unwrap();

        *counter_ref += 1;
    }

    fn free_counter(env: &Env) -> () {
        let mut counter_ref = env.free_counter.lock().unwrap();

        *counter_ref += 1;
    }

    // Create an import object.
    let import_object = imports! {
        "env" => {
            "malloc_counter" => Function::new_native_with_env(&store, shared_env.clone(), malloc_counter),
            "free_counter" => Function::new_native_with_env(&store, shared_env.clone(), free_counter),
        }
    };

    let instance = Instance::new(&module, &import_object)?;

    let wat_main = instance.exports.get_function("main")?.native::<(), i32>()?;

    wat_main.call()?;

    println!(
        "malloc counter: {:?}",
        *shared_env.malloc_counter.lock().unwrap()
    );

    println!(
        "free counter: {:?}",
        *shared_env.free_counter.lock().unwrap()
    );

    Ok(())
}

#[test]
fn test_malloc_free_counter() -> Result<(), Box<dyn std::error::Error>> {
    main()
}
