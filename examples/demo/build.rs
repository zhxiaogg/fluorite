use fluorite_codegen::code_gen::rust::RustOptions;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let options = RustOptions::new(out_dir)
        .with_any_type("serde_json::Value")
        .with_generate_new(true);

    // Compile both .fl files - order matters for imports
    fluorite_codegen::compile_fl_with_options(options, &["fluorite/common.fl", "fluorite/demo.fl"])
        .unwrap();
}
