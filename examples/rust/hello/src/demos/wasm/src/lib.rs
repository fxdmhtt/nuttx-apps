#[no_mangle]
extern "C" fn demo_wasm_hello() {
    unsafe { hello() }
}

#[link(wasm_import_module = "hello")]
extern "C" {
    fn hello();
}
