use crate::to_sql::{Dialect, ToSql};
use anyhow::Result;
use std::str::FromStr;

use crate::util::SqlExtension;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Type {
    Boolean,
    // integer types
    I16,
    I32,
    I64,
    // float types
    F32,
    F64,
    // arbitrary precision types
    Decimal,
    Numeric(u8, u8),
    // byte types
    Bytes,
    // date types
    Time,
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
    // Array types
    Array(Box<Type>),
    Other(String),
}

impl Type {
    pub fn lossy_eq(&self, other: &Type) -> bool {
        use Type::*;
        match (self, other) {
            (Other(_), _) => true,
            (a, b) => a == b,
        }
    }
}

impl FromStr for Type {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        use Type::*;
        let s = match s {
            "numeric" => Decimal,
            "bigint" => I64,
            "int8" => I64,
            "double precision" => F64,
            "real" => F32,
            "bool" => Boolean,
            "boolean" => Boolean,
            "date" => Date,
            "bytea" => Bytes,
            "timestamp with time zone" => DateTime,
            "timestamp without time zone" => NaiveDateTime,
            "interval" => Duration,
            "json" => Json,
            "jsonb" => Jsonb,
            "uuid" => Uuid,
            "smallint" => I16,
            "text" => Text,
            "character varying" => Text,
            "varchar" => Text,
            "integer" => I32,
            "ARRAY" => panic!("Encountered `ARRAY` type when reading data schema from database. ARRAY must be handled separately."),
            s => Other(s.to_string()),
        };
        Ok(s)
    }
}

impl ToSql for Type {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use self::Type::*;
        let s = match self {
            Boolean => "boolean",
            I16 => "smallint",
            I32 => "integer",
            I64 => "bigint",
            Bytes => "bytea",
            Time => "time without time zone",
            Date => "date",
            DateTime => "timestamptz",
            NaiveDateTime => "timestamp without time zone",
            Duration => "interval",
            Json => "json",
            Jsonb => "jsonb",
            F32 => "real",
            F64 => "double precision",
            Decimal => "numeric",
            Numeric(p, s) => {
                return buf.push_str(&format!("numeric({}, {})", p, s));
            }
            Uuid => "uuid",
            Text => "character varying",
            Array(inner) => {
                buf.push_sql(inner.as_ref(), dialect);
                if dialect == Dialect::Postgres {
                    buf.push_str("[]");
                } else {
                    buf.push_str(" ARRAY")
                }
                return;
            }
            Other(z) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(z, "Unknown type. SQL may not be valid.");
                buf.push_str("/* Unknown type: ");
                buf.push_str(z);
                buf.push_str(" */");
                return;
            }
        };
        buf.push_str(s);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_numeric() {
        let s = "numeric";
        let t = Type::from_str(s).unwrap();
        assert_eq!(t, Type::Decimal);
    }
}
