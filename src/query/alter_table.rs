use crate::schema::Constraint;
use crate::util::SqlExtension;
use crate::{Column, Dialect, ToSql, Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlterColumnAction {
    SetType { typ: Type, using: Option<String> },
    SetNullable(bool),
}

/// Alter table action
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlterAction {
    AddColumn {
        column: Column,
    },
    AlterColumn {
        name: String,
        action: AlterColumnAction,
    },
    AddConstraint {
        name: String,
        column: String,
        constraint: Constraint,
    },
}

impl AlterAction {
    pub fn set_nullable(name: String, nullable: bool) -> Self {
        Self::AlterColumn {
            name,
            action: AlterColumnAction::SetNullable(nullable),
        }
    }

    pub fn set_type(name: String, typ: Type) -> Self {
        Self::AlterColumn {
            name,
            action: AlterColumnAction::SetType { typ, using: None },
        }
    }

    pub fn add_constraint(table: &str, column: String, constraint: Constraint) -> Self {
        let name = format!("fk_{table}_{column}");
        Self::AddConstraint {
            name,
            column,
            constraint,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlterTable {
    pub schema: Option<String>,
    pub name: String,
    pub actions: Vec<AlterAction>,
}

impl ToSql for AlterTable {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        #[cfg(feature = "tracing")]
        tracing::error_span!(
            "alter-table",
            table = format!(
                "{}{}{}",
                self.schema.as_deref().unwrap_or(""),
                if self.schema.is_some() { "." } else { "" },
                self.name
            )
        );
        buf.push_str("ALTER TABLE ");
        buf.push_table_name(&self.schema, &self.name);
        buf.push_sql_sequence(&self.actions, ",", dialect);
    }
}

impl ToSql for AlterAction {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use AlterAction::*;
        match self {
            AddColumn { column } => {
                buf.push_str(" ADD COLUMN ");
                buf.push_str(&column.to_sql(dialect));
            }
            AlterColumn { name, action } => {
                use AlterColumnAction::*;
                buf.push_str(" ALTER COLUMN ");
                buf.push_quoted(name);
                match action {
                    SetType { typ, using } => {
                        buf.push_str(" TYPE ");
                        buf.push_sql(typ, dialect);
                        buf.push_str(" USING ");
                        if let Some(using) = using {
                            buf.push_str(&using)
                        } else {
                            buf.push_quoted(name);
                            buf.push_str("::");
                            buf.push_sql(typ, dialect);
                        }
                    }
                    SetNullable(nullable) => {
                        if *nullable {
                            buf.push_str(" DROP NOT NULL");
                        } else {
                            buf.push_str(" SET NOT NULL");
                        }
                    }
                }
            }
            AddConstraint {
                name,
                column,
                constraint,
            } => {
                buf.push_str(" ADD CONSTRAINT ");
                buf.push_quoted(name);
                buf.push_str(" FOREIGN KEY (");
                buf.push_quoted(column);
                buf.push_str(") ");
                buf.push_sql(constraint, dialect);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_alter_action() {
        let alter = AlterAction::AlterColumn {
            name: "foo".to_string(),
            action: AlterColumnAction::SetType {
                typ: Type::Text,
                using: None,
            },
        };
        assert_eq!(
            alter.to_sql(Dialect::Postgres),
            r#" ALTER COLUMN "foo" TYPE character varying USING "foo"::character varying"#
        );

        let alter = AlterAction::AlterColumn {
            name: "foo".to_string(),
            action: AlterColumnAction::SetType {
                typ: Type::Text,
                using: Some("SUBSTRING(foo, 1, 3)".to_string()),
            },
        };
        assert_eq!(
            alter.to_sql(Dialect::Postgres),
            r#" ALTER COLUMN "foo" TYPE character varying USING SUBSTRING(foo, 1, 3)"#
        );
    }
}
