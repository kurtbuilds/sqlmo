use openapiv3::{OpenAPI, SchemaKind, Schema as OaSchema, Type as OaType};
use convert_case::{Case, Casing};
use crate::schema::{Column, Table, Type};
use crate::Schema;

impl TryFrom<OpenAPI> for Schema {
    type Error = anyhow::Error;

    fn try_from(spec: OpenAPI) -> Result<Self, Self::Error> {
        let mut tables = Vec::new();
        if let Some(components) = &spec.components {
            for (schema_name, schema) in components.schemas.iter().filter(|(schema_name, schema)| {
                !schema_name.ends_with("Response")
            }) {
                let schema = schema.resolve(&spec);
                let columns = schema_to_columns(&schema, &spec)?;
                let name = schema_name.to_case(Case::Snake);
                let table = Table {
                    name: schema_name.to_case(Case::Snake),
                    columns,
                    indexes: vec![]
                };
                tables.push(table);
            }
        }
        Ok(Schema {
            tables,
        })
    }
}

fn schema_to_type(schema: &OaSchema, spec:&OpenAPI) -> anyhow::Result<Type> {
    match schema.schema_kind {
        SchemaKind::Type(OaType::String(_)) => {
            Ok(Type::Text)
        }
        SchemaKind::Type(OaType::Integer(_)) => {
            Ok(Type::Integer)
        }
        SchemaKind::Type(OaType::Boolean{..}) => {
            Ok(Type::Boolean)
        }
        SchemaKind::Type(OaType::Number(_)) => {
            Ok(Type::Numeric)
        }
        _ => panic!("Unsupported type: {:?}", schema)
    }
}

fn schema_to_columns(schema: &OaSchema, spec: &OpenAPI) -> anyhow::Result<Vec<Column>> {
    let mut columns = vec![];
    let props = schema.properties().ok_or(anyhow::anyhow!("No properties"))?;
    for (name, prop) in props.into_iter() {
        let prop = prop.resolve(spec);
        let column = Column {
            name: name.to_case(Case::Snake),
            typ: schema_to_type(prop, spec)?,
            null: prop.required(&name),
            default: None,
        };
        columns.push(column);
    }
    Ok(columns)
}