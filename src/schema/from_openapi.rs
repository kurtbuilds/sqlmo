use openapiv3::{OpenAPI, Schema as OaSchema, SchemaKind, Type as OaType};
use convert_case::{Case, Casing};
use crate::schema::Type;
use crate::Schema;
use crate::schema::column::Column;
use crate::schema::table::Table;

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
                    schema: None,
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

fn oaschema_to_sqltype(schema: &OaSchema, spec:&OpenAPI) -> anyhow::Result<Type> {
    use Type::*;
    let s = match schema.schema_kind {
        SchemaKind::Type(OaType::String(_)) => {
            Text
        }
        SchemaKind::Type(OaType::Integer(_)) => {
            Integer
        }
        SchemaKind::Type(OaType::Boolean{..}) => {
            Boolean
        }
        SchemaKind::Type(OaType::Number(_)) => {
            Numeric
        }
        _ => panic!("Unsupported type: {:?}", schema)
    };
    Ok(s)
}

fn schema_to_columns(schema: &OaSchema, spec: &OpenAPI) -> anyhow::Result<Vec<Column>> {
    let mut columns = vec![];
    let props = schema.properties().ok_or(anyhow::anyhow!("No properties"))?;
    for (name, prop) in props.into_iter() {
        let prop = prop.resolve(spec);
        let column = Column {
            primary_key: false,
            name: name.to_case(Case::Snake),
            typ: oaschema_to_sqltype(prop, spec)?,
            nullable: prop.required(&name),
            default: None,
        };
        columns.push(column);
    }
    Ok(columns)
}