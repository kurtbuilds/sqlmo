use std::collections::HashMap;
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Schema {
    pub tables: Vec<Table>,
}

impl Schema {
    pub fn new() -> Schema {
        Schema {
            tables: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Boolean,
    // integer types
    SmallInt,
    BigInt,
    Integer,
    // float types
    Numeric,
    // byte types
    Bytes,
    // date types
    Date,
    DateTime,
    NaiveDateTime,
    Duration,
    // json types
    Json,
    Jsonb,
    // extension types
    Uuid,
    // string types
    Text,
}

impl Type {
    pub fn from_str(s: &str) -> Result<Type> {
        match s {
            "bigint" => Ok(Type::BigInt),
            "boolean" => Ok(Type::Boolean),
            "date" => Ok(Type::Date),
            "bytea" => Ok(Type::Bytes),
            "timestamp with time zone" => Ok(Type::DateTime),
            "timestamp without time zone" => Ok(Type::NaiveDateTime),
            "interval" => Ok(Type::Duration),
            "json" => Ok(Type::Json),
            "jsonb" => Ok(Type::Jsonb),
            "numeric" => Ok(Type::Numeric),
            "uuid" => Ok(Type::Uuid),
            "smallint" => Ok(Type::SmallInt),
            "text" => Ok(Type::Text),
            "character varying" => Ok(Type::Text),
            "integer" => Ok(Type::Integer),
            _ => Err(anyhow!("Unknown type: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub typ: Type,
    pub nullable: bool,
    pub primary_key: bool,
    pub default: Option<String>,
}

#[derive(Debug)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
}

pub trait ToSql {
    fn to_sql(&self) -> String;
}

impl ToSql for Type {
    fn to_sql(&self) -> String {
        use self::Type::*;
        match self {
            BigInt => "bigint".to_string(),
            Boolean => "boolean".to_string(),
            Bytes => "bytea".to_string(),
            Date => "date".to_string(),
            DateTime => "timestamptz".to_string(),
            NaiveDateTime => "timestamp without time zone".to_string(),
            Duration => "interval".to_string(),
            Json => "json".to_string(),
            Jsonb => "jsonb".to_string(),
            Numeric => "numeric".to_string(),
            SmallInt => "smallint".to_string(),
            Uuid => "uuid".to_string(),
            Integer => "integer".to_string(),
            Text => "character varying".to_string(),
        }
    }
}
