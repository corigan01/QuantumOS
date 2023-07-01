use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=linker.ld");

    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!(
        "cargo:rustc-link-arg-bins=-script={}",
        local_path.join("linker.ld").display()
    )
}
