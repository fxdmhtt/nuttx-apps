use std::ffi::c_char;

#[no_mangle]
extern "C" fn demo_wasm_hello() {
    println!("[WASM] Hello World from WAMR module written in Rust!");
    unsafe { hello() };

    let x = (1..=9)
        .map(|x| x * 10)
        .filter(|x| x % 2 == 0)
        .map(|x| x / 10)
        .collect::<Vec<_>>()
        .into_iter()
        .reduce(|acc, x| acc * 10 + x)
        .unwrap();
    if format!("{x}") == "123456789" {
        unsafe { println(c"Hello World written in Rust from WAMR module!".as_ptr()) };
    }
}

#[link(wasm_import_module = "hello")]
extern "C" {
    fn hello();
    fn println(s: *const c_char);
}
