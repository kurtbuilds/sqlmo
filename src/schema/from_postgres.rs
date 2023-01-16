use anyhow::{Error, Result};
use itertools::Itertools;
use sqlx::PgConnection;
use sqlx::postgres::PgPoolOptions;

use crate::{Schema, schema};
use crate::schema::column::Column;
use crate::schema::table::Table;

const QUERY_COLUMNS: &str = "SELECT
	table_name, column_name, ordinal_position, is_nullable, data_type
FROM information_schema.columns
WHERE
	table_schema=$1
ORDER BY table_name, ordinal_position";

const QUERY_TABLES: &str = "SELECT
    table_schema
    , table_name
FROM information_schema.tables
WHERE
    table_schema = $1 ";

#[derive(sqlx::FromRow)]
pub struct SchemaColumn {
    pub table_name: String,
    pub column_name: String,
    pub ordinal_position: i32,
    pub is_nullable: String,
    pub data_type: String,
}

pub async fn query_schema_columns(mut conn: &mut PgConnection, schema_name: &str) -> Result<Vec<SchemaColumn>> {
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

pub async fn query_table_names(mut conn: &mut PgConnection, schema_name: &str) -> Result<Vec<String>> {
    let result = sqlx::query_as::<_, TableSchema>(QUERY_TABLES)
        .bind(schema_name)
        .fetch_all(conn)
        .await?;
    Ok(result.into_iter().map(|t| t.table_name).collect())
}


impl Schema {
    pub async fn try_from_database(conn: &mut PgConnection, schema_name: &str) -> Result<Schema> {
        let column_schemas = query_schema_columns(conn, schema_name).await?;
        let mut tables = column_schemas.into_iter()
            .group_by(|c| c.table_name.clone())
            .into_iter()
            .map(|(table_name, group)| {
                let columns = group.map(|c: SchemaColumn| {
                    let nullable = c.is_nullable == "YES";
                    let typ = schema::Type::from_str(&c.data_type)?;
                    Ok(Column {
                        name: c.column_name.clone(),
                        typ,
                        nullable,
                        primary_key: false,
                        default: None,
                    })
                }).collect::<Result<Vec<_>, Error>>()?;
                Ok(Table {
                    schema: None,
                    name: table_name,
                    columns,
                    indexes: vec![]
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