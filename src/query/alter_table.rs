use crate::{Dialect, Column, ToSql, Type};
use crate::util::SqlExtension;

#[derive(Debug)]
pub enum AlterColumnAction {
    SetType(Type),
    SetNullable(bool),
}

/// Alter table action
#[derive(Debug)]
pub enum AlterAction {
    AddColumn {
        column: Column,
    },
    AlterColumn {
        name: String,
        action: AlterColumnAction,
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
            action: AlterColumnAction::SetType(typ),
        }
    }
}

#[derive(Debug)]
pub struct AlterTable {
    pub schema: Option<String>,
    pub name: String,
    pub actions: Vec<AlterAction>,
}

impl ToSql for AlterTable {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        #[cfg(feature = "tracing")]
        tracing::error_span!("alter-table", table = format!(
            "{}{}{}",
            self.schema.as_deref().unwrap_or(""),
            if self.schema.is_some() { "." } else { "" },
            self.name
        ));
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
                    SetType(ty) => {
                        buf.push_str(" TYPE ");
                        buf.push_sql(ty, dialect);
                        buf.push_str(" USING ");
                        buf.push_quoted(name);
                        buf.push_str("::");
                        buf.push_sql(ty, dialect);
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
        }
    }
}