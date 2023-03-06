use std::str::FromStr;
use anyhow::{Error, Result};
use itertools::Itertools;
use sqlx::PgConnection;

use crate::{Schema, schema};
use crate::schema::column::Column;
use crate::schema::table::Table;

const QUERY_COLUMNS: &str = include_str!("from_postgres/query_columns.sql");
const QUERY_TABLES: &str = include_str!("from_postgres/query_tables.sql");

#[derive(sqlx::FromRow)]
pub struct SchemaColumn {
    pub table_name: String,
    pub column_name: String,
    pub ordinal_position: i32,
    pub is_nullable: String,
    pub data_type: String,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub inner_type: Option<String>,
}

pub async fn query_schema_columns(conn: &mut PgConnection, schema_name: &str) -> Result<Vec<SchemaColumn>> {
    let result = sqlx::query_as::<_, SchemaColumn>(QUERY_COLUMNS)
        .bind(schema_name)
        .fetch_all(conn)
        .await?;
    Ok(result)
}

#[derive(sqlx::FromRow)]
pub struct TableSchema {
    pub table_schema: String,
    pub table_name: String,
}

pub async fn query_table_names(conn: &mut PgConnection, schema_name: &str) -> Result<Vec<String>> {
    let result = sqlx::query_as::<_, TableSchema>(QUERY_TABLES)
        .bind(schema_name)
        .fetch_all(conn)
        .await?;
    Ok(result.into_iter().map(|t| t.table_name).collect())
}


impl TryInto<Column> for SchemaColumn {
    type Error = Error;

    fn try_into(self) -> std::result::Result<Column, Self::Error> {
        use schema::Type::*;
        let nullable = self.is_nullable == "YES";
        let typ = match self.data_type.as_str() {
            "ARRAY" => {
                let inner = schema::Type::from_str(&self.inner_type.expect("Encounterd ARRAY with no inner type."))?;
                Array(Box::new(inner))
            }
            "numeric" if self.numeric_precision.is_some() && self.numeric_scale.is_some() => {
                Numeric(self.numeric_precision.unwrap() as u8, self.numeric_scale.unwrap() as u8)
            }
            z => schema::Type::from_str(z)?,
        };
        Ok(Column {
            name: self.column_name.clone(),
            typ,
            nullable,
            primary_key: false,
            default: None,
        })
    }
}

impl Schema {
    pub async fn try_from_database(conn: &mut PgConnection, schema_name: &str) -> Result<Schema> {
        let column_schemas = query_schema_columns(conn, schema_name).await?;
        let mut tables = column_schemas.into_iter()
            .group_by(|c| c.table_name.clone())
            .into_iter()
            .map(|(table_name, group)| {
                let columns = group
                    .map(|c: SchemaColumn| c.try_into())
                    .collect::<Result<Vec<_>, Error>>()?;
                Ok(Table {
                    schema: None,
                    name: table_name,
                    columns,
                    indexes: vec![],
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        // Degenerate case but you can have tables with no columns...
        let table_names = query_table_names(conn, schema_name).await?;
        for name in table_names {
            if tables.iter().any(|t| t.name == name) {
                continue;
            }
            tables.push(Table {
                schema: None,
                name,
                columns: vec![],
                indexes: vec![],
            })
        }
        Ok(Schema { tables })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_numeric() {
        let c = SchemaColumn {
            table_name: "foo".to_string(),
            column_name: "bar".to_string(),
            ordinal_position: 1,
            is_nullable: "NO".to_string(),
            data_type: "numeric".to_string(),
            numeric_precision: Some(10),
            numeric_scale: Some(2),
            inner_type: None,
        };
        let column: Column = c.try_into().unwrap();
        assert_eq!(column.typ, schema::Type::Numeric(10, 2));
    }

    #[test]
    fn test_integer() {
        let c = SchemaColumn {
            table_name: "foo".to_string(),
            column_name: "bar".to_string(),
            ordinal_position: 1,
            is_nullable: "NO".to_string(),
            data_type: "integer".to_string(),
            numeric_precision: Some(32),
            numeric_scale: Some(0),
            inner_type: None,
        };
        let column: Column = c.try_into().unwrap();
        assert_eq!(column.typ, schema::Type::I32);
    }
}