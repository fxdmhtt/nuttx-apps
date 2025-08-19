mod binding;
mod demos;
mod runtime;

// Function hello_rust_cargo without manglng
// Bug: Cannot run twice because of random exceptions in tokio and infinite recursion stack overflow
#[no_mangle]
pub extern "C" fn hello_rust_cargo_main() {
    demos::serde::demo_serde();

    {
        let handle = demos::tokio::demo_thread(30);
        demos::tokio::demo_tokio(30);
        handle.join().unwrap();
    }
}
