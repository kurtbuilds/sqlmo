use openapiv3::{OpenAPI, Schema as OaSchema, SchemaKind, Type as OaType};
use convert_case::{Case, Casing};
use crate::schema::Type;
use crate::Schema;
use crate::schema::column::Column;
use crate::schema::table::Table;

#[derive(Debug, Clone)]
pub struct FromOpenApiOptions {
    pub include_arrays: bool,
}

impl Default for FromOpenApiOptions {
    fn default() -> Self {
        Self {
            include_arrays: false,
        }
    }
}

impl Schema {
    pub fn try_from_openapi(spec: OpenAPI, options: &FromOpenApiOptions) -> anyhow::Result<Self> {
        let mut tables = Vec::new();
        if let Some(components) = &spec.components {
            for (schema_name, schema) in components.schemas.iter().filter(|(schema_name, _)| {
                !schema_name.ends_with("Response")
            }) {
                let schema = schema.resolve(&spec);
                let Some(columns) =  schema_to_columns(&schema, &spec, options)? else {
                    continue
                };
                let table = Table {
                    schema: None,
                    name: schema_name.to_case(Case::Snake),
                    columns,
                    indexes: vec![],
                };
                tables.push(table);
            }
        }
        Ok(Schema {
            tables,
        })
    }
}

fn oaschema_to_sqltype(schema: &OaSchema, _: &OpenAPI, options: &FromOpenApiOptions) -> anyhow::Result<Option<Type>> {
    use Type::*;
    let s = match schema.schema_kind {
        SchemaKind::Type(OaType::String(_)) => {
            Text
        }
        SchemaKind::Type(OaType::Integer(_)) => {
            Integer
        }
        SchemaKind::Type(OaType::Boolean { .. }) => {
            Boolean
        }
        SchemaKind::Type(OaType::Number(_)) => {
            Numeric
        }
        SchemaKind::Type(OaType::Array(_)) => {
            if options.include_arrays {
                Jsonb
            } else {
                return Ok(None);
            }
        }
        SchemaKind::Type(OaType::Object(_)) => {
            Jsonb
        }
        _ => panic!("Unsupported type: {:#?}", schema)
    };
    Ok(Some(s))
}

fn schema_to_columns(schema: &OaSchema, spec: &OpenAPI, options: &FromOpenApiOptions) -> anyhow::Result<Option<Vec<Column>>> {
    let mut columns = vec![];
    let Some(props) = schema.properties() else {
        return Ok(None);
    };
    for (name, prop) in props.into_iter() {
        let prop = prop.resolve(spec);
        let typ = oaschema_to_sqltype(prop, spec, options)?;
        let Some(typ) = typ else {
            continue;
        };
        let column = Column {
            primary_key: false,
            name: name.to_case(Case::Snake),
            typ,
            nullable: prop.required(&name),
            default: None,
        };
        columns.push(column);
    }
    Ok(Some(columns))
}
