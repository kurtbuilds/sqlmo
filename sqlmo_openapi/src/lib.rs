use convert_case::{Case, Casing};
use sqlmo::{Column, Schema, Table, Type};
use sqlmo::util::pkey_column_names;

use openapiv3 as oa;

pub trait FromOpenApi: Sized {
    fn try_from_openapi(spec: openapiv3::OpenAPI, options: &FromOpenApiOptions) -> anyhow::Result<Self>;
}

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

impl FromOpenApi for Schema {
    fn try_from_openapi(spec: oa::OpenAPI, options: &FromOpenApiOptions) -> anyhow::Result<Self> {
        let mut tables = Vec::new();
        if let Some(components) = &spec.components {
            for (schema_name, schema) in components.schemas.iter().filter(|(schema_name, _)| {
                !schema_name.ends_with("Response")
            }) {
                let schema = schema.resolve(&spec);
                let Some(mut columns) = schema_to_columns(&schema, &spec, options)? else {
                    continue;
                };
                let pkey_candidates = pkey_column_names(&schema_name);
                for col in &mut columns {
                    if pkey_candidates.contains(&col.name) {
                        col.primary_key = true;
                        break;
                    }
                }
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

fn oaschema_to_sqltype(schema: &oa::Schema, options: &FromOpenApiOptions) -> anyhow::Result<Option<Type>> {
    use sqlmo::Type::*;
    let s = match &schema.schema_kind {
        oa::SchemaKind::Type(oa::Type::String(s)) => {
            match s.format.as_str() {
                "currency" => Numeric(19, 4),
                "decimal" => Decimal,
                "date" => Date,
                "date-time" => DateTime,
                _ => Text,
            }
        }
        oa::SchemaKind::Type(oa::Type::Integer(_)) => {
            let format = schema.schema_data.extensions.get("x-format").and_then(|v| v.as_str());
            match format {
                Some("date") => Date,
                _ => I64,
            }
        }
        oa::SchemaKind::Type(oa::Type::Boolean { .. }) => {
            Boolean
        }
        oa::SchemaKind::Type(oa::Type::Number(_)) => {
            F64
        }
        oa::SchemaKind::Type(oa::Type::Array(_)) => {
            if options.include_arrays {
                Jsonb
            } else {
                return Ok(None);
            }
        }
        oa::SchemaKind::Type(oa::Type::Object(_)) => {
            Jsonb
        }
        _ => panic!("Unsupported type: {:#?}", schema)
    };
    Ok(Some(s))
}

fn schema_to_columns(schema: &oa::Schema, spec: &oa::OpenAPI, options: &FromOpenApiOptions) -> anyhow::Result<Option<Vec<Column>>> {
    let mut columns = vec![];
    let Some(props) = schema.properties() else {
        return Ok(None);
    };
    for (name, prop) in props.into_iter() {
        let prop = prop.resolve(spec);
        let typ = oaschema_to_sqltype(prop, options)?;
        let Some(typ) = typ else {
            continue;
        };
        let mut primary_key = false;
        if name == "id" {
            primary_key = true;
        }
        let mut nullable = true;
        if primary_key {
            nullable = false;
        }
        if prop.required(&name) {
            nullable = false;
        }
        if prop.schema_data.extensions.get("x-format").and_then(|v| v.as_str()) == Some("date") {
            nullable = true;
        }
        if prop.schema_data.extensions.get("x-null-as-zero").and_then(|v| v.as_bool()).unwrap_or(false) {
            nullable = true;
        }
        let column = Column {
            primary_key,
            name: name.to_case(Case::Snake),
            typ,
            nullable,
            default: None,
        };
        columns.push(column);
    }
    Ok(Some(columns))
}


#[cfg(test)]
mod test {
    use openapiv3::OpenAPI;

    use super::*;

    use openapiv3 as oa;

    #[test]
    fn test_format_date() {
        let mut z = oa::Schema::new_object();

        let mut int_format_date = oa::Schema::new_integer();
        int_format_date.schema_data.extensions.insert("x-format".to_string(), serde_json::Value::from("date"));
        z.add_property("date", int_format_date).unwrap();

        let mut int_null_as_zero = oa::Schema::new_integer();
        int_null_as_zero.schema_data.extensions.insert("x-null-as-zero".to_string(), serde_json::Value::from(true));
        z.add_property("int_null_as_zero", int_null_as_zero).unwrap();

        let columns = schema_to_columns(&z, &OpenAPI::default(), &FromOpenApiOptions::default()).unwrap().unwrap();
        assert_eq!(columns.len(), 2);

        let int_format_date = &columns[0];
        assert_eq!(int_format_date.name, "date");
        assert_eq!(int_format_date.nullable, true);

        let int_null_as_zero = &columns[1];
        assert_eq!(int_null_as_zero.name, "int_null_as_zero");
        assert_eq!(int_null_as_zero.nullable, true);
    }

    #[test]
    fn test_oasformat() {
        let z = oa::Schema::new_string().with_format("currency");
        let t = oaschema_to_sqltype(&z, &FromOpenApiOptions::default()).unwrap().unwrap();
        assert_eq!(t, Type::Numeric(19, 4));

        let z = oa::Schema::new_string().with_format("decimal");
        let t = oaschema_to_sqltype(&z, &FromOpenApiOptions::default()).unwrap().unwrap();
        assert_eq!(t, Type::Decimal);
    }
}