use std::{collections::HashMap, io::Write};

use crate::definitions::Definition;

mod type_info;
pub use type_info::*;

pub trait CodeGenProvider<C: CodeGenContext> {
    fn get_pre_processor(&self) -> Box<dyn PreProcessor<C>>;
    fn get_package_writer(&self) -> Option<Box<dyn PackageWriter<C>>>;
    fn get_object_writer(&self) -> Box<dyn ObjectWriter<C>>;
    fn get_enum_writer(&self) -> Box<dyn EnumWriter<C>>;
    fn get_object_enum_writer(&self) -> Box<dyn ObjectEnumWriter<C>>;
    fn get_list_writer(&self) -> Box<dyn ListWriter<C>>;
    fn get_map_writer(&self) -> Box<dyn MapWriter<C>>;
}

pub trait CodeGenContext {
    fn type_dict(&self) -> &HashMap<String, TypeInfo>;
    fn get_writer_for_type(&self, type_info: &TypeInfo) -> anyhow::Result<Box<dyn Write>>;
}

pub trait PreProcessor<C: CodeGenContext> {
    fn process(&self, definitions: HashMap<String, TypeInfo>) -> anyhow::Result<Box<C>>;
    // TODO: move this to extra type info within context?
    fn get_package_name(&self, definition: &Definition) -> anyhow::Result<String>;
}

pub trait PackageWriter<C: CodeGenContext> {
    fn write_package(
        &self,
        package: &str,
        types: &Vec<&TypeInfo>,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait MapWriter<C: CodeGenContext> {
    fn write_map(
        &self,
        writer: &mut dyn Write,
        type_info: &MapTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait ListWriter<C: CodeGenContext> {
    fn write_list(
        &self,
        writer: &mut dyn Write,
        type_info: &ListTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait ObjectEnumWriter<C: CodeGenContext> {
    fn write_object_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &ObjectEnumTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait EnumWriter<C: CodeGenContext> {
    fn write_enum(
        &self,
        writer: &mut dyn Write,
        type_info: &EnumTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}

pub trait ObjectWriter<C: CodeGenContext> {
    fn write_object(
        &self,
        writer: &mut dyn Write,
        type_info: &ObjectTypeInfo,
        context: &C,
    ) -> anyhow::Result<()>;
}
