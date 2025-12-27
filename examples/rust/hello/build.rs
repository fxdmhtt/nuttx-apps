#[path = "tools/i18n.rs"]
mod i18n;

fn main() {
    i18n::generate();

    println!("cargo:rerun-if-changed=build/");
    println!("cargo:rerun-if-changed=build.rs");
}
