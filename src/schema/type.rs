use std::str::FromStr;
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

impl FromStr for Type {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
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
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        use self::Type::*;
        let s = match self {
            BigInt => "bigint",
            Boolean => "boolean",
            Bytes => "bytea",
            Date => "date",
            DateTime => "timestamptz",
            NaiveDateTime => "timestamp without time zone",
            Duration => "interval",
            Json => "json",
            Jsonb => "jsonb",
            Numeric => "numeric",
            SmallInt => "smallint",
            Uuid => "uuid",
            Integer => "integer",
            Text => "character varying",
        };
        buf.push_str(s);
    }
}