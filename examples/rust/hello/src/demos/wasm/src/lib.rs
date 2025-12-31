use std::ffi::{c_char, CString};

#[no_mangle]
extern "C" fn demo_wasm_hello() {
    println!("Hello World from WAMR module written in Rust!");
    unsafe { hello() };

    let x = (1..=9)
        .map(|x| x * 10)
        .filter(|x| x % 2 == 0)
        .map(|x| x / 10)
        .collect::<Vec<_>>()
        .into_iter()
        .reduce(|acc, x| acc * 10 + x)
        .unwrap();
    let s = CString::new(format!("{x}")).unwrap();
    unsafe { hello_printf(s.as_ptr()) };
}

#[link(wasm_import_module = "hello")]
extern "C" {
    fn hello();
    fn hello_printf(s: *const c_char);
}
