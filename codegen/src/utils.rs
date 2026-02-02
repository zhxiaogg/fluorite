use std::fs;
use std::sync::Arc;

use crate::code_gen::fs::RealFileSystem;
use crate::code_gen::rust::{RustOptions, RustTemplateGenerator};
use crate::definitions::Definition;
use crate::idl::parse_to_ir;

/// Compile YAML definitions to Rust code with custom options
pub fn compile_with_options(options: RustOptions, yaml_files: &[&str]) -> anyhow::Result<()> {
    let definitions: Vec<Definition> = yaml_files
        .iter()
        .map(|path| {
            let content = fs::read_to_string(path)?;
            let def: Definition = serde_yaml::from_str(&content)?;
            Ok(def)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let fs = Arc::new(RealFileSystem::new());
    let generator = RustTemplateGenerator::new(options, fs);
    generator.generate(&definitions)?;

    Ok(())
}

/// Compile Fluorite IDL (.fl) files to Rust code with custom options
///
/// # Example
///
/// ```rust,no_run
/// use fluorite_codegen::code_gen::rust::RustOptions;
///
/// let out_dir = std::env::var("OUT_DIR").unwrap();
/// let options = RustOptions::new(out_dir);
/// fluorite_codegen::compile_fl_with_options(options, &["common.fl", "demo.fl"]).unwrap();
/// ```
pub fn compile_fl_with_options(options: RustOptions, fl_files: &[&str]) -> anyhow::Result<()> {
    let schema = parse_to_ir(fl_files)?;

    let fs = Arc::new(RealFileSystem::new());
    let generator = RustTemplateGenerator::new(options, fs);
    generator.generate_from_schema(&schema)?;

    Ok(())
}

/// Compile YAML definitions to Rust code with default options
pub fn compile(output_dir: &str, yaml_files: &[&str]) -> anyhow::Result<()> {
    let options = RustOptions::new(output_dir.to_string());
    compile_with_options(options, yaml_files)
}

pub(crate) fn deserialize_definition_file(file_path: &str) -> anyhow::Result<Definition> {
    let file_content = fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}

#[cfg(test)]
mod test {
    use super::deserialize_definition_file;

    #[test]
    fn test_deserialize_definition_file() -> anyhow::Result<()> {
        deserialize_definition_file("tests/fixtures/users.yml")?;
        Ok(())
    }
}
