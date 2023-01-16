use anyhow::{anyhow, Result};
use crate::to_sql::{Dialect, ToSql};

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
    pub fn from_str(s: &str) -> Result<Self> {
        use Type::*;
        let s = match s {
            "bigint" => BigInt,
            "boolean" => Boolean,
            "date" => Date,
            "bytea" => Bytes,
            "timestamp with time zone" => DateTime,
            "timestamp without time zone" => NaiveDateTime,
            "interval" => Duration,
            "json" => Json,
            "jsonb" => Jsonb,
            "numeric" => Numeric,
            "uuid" => Uuid,
            "smallint" => SmallInt,
            "text" => Text,
            "character varying" => Text,
            "integer" => Integer,
            _ => return Err(anyhow!("Unknown type: {}", s)),
        };
        Ok(s)
    }
}

impl ToSql for Type {
    fn to_sql(&self, _dialect: Dialect) -> String {
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