use crate::code_gen::abi::{
    CodeGenContext, EnumTypeInfo, EnumWriter, ListTypeInfo, ListWriter, MapTypeInfo, MapWriter,
    ObjectEnumTypeInfo, ObjectEnumValue, ObjectEnumWriter, ObjectField, ObjectTypeInfo,
    ObjectWriter, TypeInfo,
};

use super::RustContext;
use std::io::Write;

use anyhow::anyhow;

pub struct RustTypeWriter {}

impl ObjectWriter<RustContext> for RustTypeWriter {
    fn write_object(
        &self,
        writer: &mut dyn Write,
        type_info: &ObjectTypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        self.pre_write_type(writer)?;
        writer.write_all("#[derive(Debug, Clone, Serialize, Deserialize)]\n".as_bytes())?;
        writer.write_all(format!("pub struct {} {{\n", type_info.name).as_bytes())?;
        // write fields
        for field in type_info.fields.iter() {
            self.write_object_field(writer, field, type_info, context)?;
        }
        writer.write_all("}\n".as_bytes())?;
        Ok(())
    }
}

impl EnumWriter<RustContext> for RustTypeWriter {
    fn write_enum(
        &self,
        writer: &mut dyn Write,
        enum_type_info: &EnumTypeInfo,
        _context: &RustContext,
    ) -> anyhow::Result<()> {
        self.pre_write_type(writer)?;
        writer.write_all("#[derive(Debug, Clone, Serialize, Deserialize)]\n".as_bytes())?;
        writer.write_all(format!("pub enum {} {{\n", enum_type_info.name).as_bytes())?;
        // write values
        for value in enum_type_info.values.iter() {
            writer.write_all(format!("  {},\n", value).as_bytes())?;
        }
        writer.write_all("}".as_bytes())?;

        Ok(())
    }
}

impl ObjectEnumWriter<RustContext> for RustTypeWriter {
    fn write_object_enum(
        &self,
        writer: &mut dyn Write,
        object_enum_type_info: &ObjectEnumTypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        self.pre_write_type(writer)?;
        writer.write_all("#[derive(Debug, Clone, Serialize, Deserialize)]\n".as_bytes())?;
        writer.write_all(format!("#[serde(tag = \"{}\")]\n", object_enum_type_info.type_tag).as_bytes())?;
        writer.write_all(format!("pub enum {} {{\n", object_enum_type_info.name).as_bytes())?;
        for value in object_enum_type_info.values.iter() {
            match value {
                ObjectEnumValue::Simple(simple) => {
                    writer.write_all(format!("  {},\n", simple).as_bytes())?;
                }
                ObjectEnumValue::CustomType(type_name) => {
                    match context.type_dict().get(type_name) {
                        Some(TypeInfo::Object(type_info)) => {
                            writer.write_all(format!("  {} {{\n", type_info.name).as_bytes())?;
                            for field in &type_info.fields {
                                self.write_object_field(writer, field, type_info, context)?;
                            }
                            writer.write_all("  },\n".as_bytes())?;
                        }
                        _ => {
                            return Err(anyhow!(
                                "Enum cannot be nested within enum object: {}",
                                type_name
                            ));
                        }
                    }
                }
            }
        }
        writer.write_all("}".as_bytes())?;

        Ok(())
    }
}

impl MapWriter<RustContext> for RustTypeWriter {
    fn write_map(
        &self,
        writer: &mut dyn Write,
        type_info: &MapTypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        let value_type = context.get_fully_qualified_type_name(&type_info.value_type)?;
        let key_type = context.get_fully_qualified_type_name(&type_info.key_type)?;

        writer.write_all("use std::collections::HashMap;\n\n".as_bytes())?;
        writer.write_all(
            format!(
                "pub type {} = HashMap<{}, {}>;\n",
                type_info.name, key_type, value_type
            )
            .as_bytes(),
        )?;
        Ok(())
    }
}
impl ListWriter<RustContext> for RustTypeWriter {
    fn write_list(
        &self,
        writer: &mut dyn Write,
        type_info: &ListTypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        let item_type = context.get_fully_qualified_type_name(&type_info.item_type)?;
        writer
            .write_all(format!("pub type {} = Vec<{}>;\n", type_info.name, item_type).as_bytes())?;
        Ok(())
    }
}

impl RustTypeWriter {
    fn write_object_field(
        &self,
        writer: &mut dyn Write,
        field: &ObjectField,
        type_info: &ObjectTypeInfo,
        context: &RustContext,
    ) -> anyhow::Result<()> {
        let type_to_write = context.get_fully_qualified_type_name(&field.field_type)?;
        let type_to_write = match field.is_optional() {
            true => format!("Option<{}>", type_to_write),
            _ => type_to_write,
        };
        let type_to_write = match &field
            .config
            .as_ref()
            .and_then(|c| c.rust_type_wrapper.clone())
        {
            Some(_) => format!("Box<{}>", type_to_write),
            _ => type_to_write,
        };
        match &field.config.as_ref().and_then(|c| c.rename.clone()) {
            Some(rename) => {
                if type_info.is_object_enum_value {
                    writer.write_all(
                        format!("    #[serde(rename = \"{}\")]\n", field.name).as_bytes(),
                    )?;
                    writer.write_all(format!("    {}: {},\n", rename, type_to_write).as_bytes())?;
                } else {
                    writer.write_all(
                        format!("  #[serde(rename = \"{}\")]\n", field.name).as_bytes(),
                    )?;
                    writer
                        .write_all(format!("  pub {}: {},\n", rename, type_to_write).as_bytes())?;
                }
            }
            None => {
                if type_info.is_object_enum_value {
                    writer.write_all(
                        format!("    {}: {},\n", field.name, type_to_write).as_bytes(),
                    )?;
                } else {
                    writer.write_all(
                        format!("  pub {}: {},\n", field.name, type_to_write).as_bytes(),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn pre_write_type(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        writer.write_all("use serde::{Serialize, Deserialize};".as_bytes())?;
        writer.write_all("\n\n".as_bytes())?;
        Ok(())
    }
}
