use std::str::FromStr;
use anyhow::{anyhow, Result};
use crate::to_sql::{Dialect, ToSql};

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Boolean,
    // integer types
    SmallInt,
    BigInt,
    Integer,
    // float types
    Float64,
    Numeric(u8, u8),
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
        lazy_static! {
            static ref NUMERIC_RE: Regex = Regex::new(r"numeric\((\d+), (\d+)\)").unwrap();
        }
        let cap = NUMERIC_RE.captures(s);
        if let Some(cap) = cap {
            let p = cap.get(1).unwrap().as_str().parse()?;
            let s = cap.get(2).unwrap().as_str().parse()?;
            return Ok(Numeric(p, s));
        }
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
            "numeric" => Float64,
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
            Float64 => "numeric",
            Numeric(p, s) => return buf.push_str(&format!("numeric({}, {})", p, s)),
            SmallInt => "smallint",
            Uuid => "uuid",
            Integer => "integer",
            Text => "character varying",
        };
        buf.push_str(s);
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_numeric() {
        let s = "numeric(15, 2)";
        let t = Type::from_str(s).unwrap();
        assert_eq!(t, Type::Numeric(15, 2));
    }
}