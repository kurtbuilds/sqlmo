use crate::util::SqlExtension;
use crate::{Dialect, ToSql};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Case {
    cases: Vec<(Expr, Expr)>,
    els: Option<Box<Expr>>,
}

impl Case {
    pub fn new_when<C: Into<Expr>, V: Into<Expr>>(condition: C, then_value: V) -> Self {
        Self {
            cases: vec![(condition.into(), then_value.into())],
            els: None,
        }
    }

    pub fn when(mut self, condition: Expr, value: Expr) -> Self {
        self.cases.push((condition, value));
        self
    }

    pub fn els<V: Into<Expr>>(mut self, value: V) -> Self {
        self.els = Some(Box::new(value.into()));
        self
    }
}

impl ToSql for Case {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        buf.push_str("CASE ");
        for c in &self.cases {
            buf.push_str("WHEN ");
            buf.push_sql(&c.0, dialect);
            buf.push_str(" THEN ");
            buf.push_sql(&c.1, dialect);
        }
        if let Some(els) = &self.els {
            buf.push_str(" ELSE ");
            buf.push_sql(els.as_ref(), dialect);
        }
        buf.push_str(" END");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Operation {
    Eq,
    Gte,
    Lte,
    Gt,
    Lt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Expr {
    Case(Case),
    And(Vec<Expr>),
    Raw(String),
    NotDistinctFrom(Box<Expr>, Box<Expr>),
    Column {
        schema: Option<String>,
        table: Option<String>,
        column: String,
    },
    BinOp(Operation, Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn excluded(column: &str) -> Self {
        Self::Raw(format!("excluded.\"{}\"", column))
    }

    pub fn column(column: &str) -> Self {
        Self::Column {
            schema: None,
            table: None,
            column: column.to_string(),
        }
    }

    pub fn new_eq<L: Into<Expr>, R: Into<Expr>>(left: L, right: R) -> Self {
        Self::BinOp(Operation::Eq, Box::new(left.into()), Box::new(right.into()))
    }

    pub fn table_column(table: &str, column: &str) -> Self {
        Self::Column {
            schema: None,
            table: Some(table.to_string()),
            column: column.to_string(),
        }
    }

    pub fn schema_column(schema: &str, table: &str, column: &str) -> Self {
        Self::Column {
            schema: Some(schema.to_string()),
            table: Some(table.to_string()),
            column: column.to_string(),
        }
    }

    pub fn new_and(and: Vec<Expr>) -> Self {
        Self::And(and)
    }

    pub fn case(case: Case) -> Self {
        Self::Case(case)
    }

    pub fn not_distinct_from<L: Into<Expr>, R: Into<Expr>>(left: L, right: R) -> Self {
        Self::NotDistinctFrom(Box::new(left.into()), Box::new(right.into()))
    }
}

impl Into<Expr> for &str {
    fn into(self) -> Expr {
        Expr::Raw(self.to_string())
    }
}

impl ToSql for Expr {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        match self {
            Expr::Case(c) => c.write_sql(buf, dialect),
            Expr::And(and) => {
                buf.push('(');
                buf.push_sql_sequence(&and, " AND ", dialect);
                buf.push(')');
            }
            Expr::Raw(a) => buf.push_str(a),
            Expr::NotDistinctFrom(l, r) => {
                buf.push_sql(l.as_ref(), dialect);
                buf.push_str(" IS NOT DISTINCT FROM ");
                buf.push_sql(r.as_ref(), dialect);
            }
            Expr::Column {
                schema,
                table,
                column,
            } => {
                if let Some(schema) = schema {
                    buf.push_quoted(schema);
                    buf.push('.');
                }
                if let Some(table) = table {
                    buf.push_quoted(table);
                    buf.push('.');
                }
                buf.push_quoted(column);
            }
            Expr::BinOp(op, l, r) => {
                buf.push_sql(l.as_ref(), dialect);
                buf.push_sql(op, dialect);
                buf.push_sql(r.as_ref(), dialect);
            }
        }
    }
}

impl ToSql for Operation {
    fn write_sql(&self, buf: &mut String, _dialect: Dialect) {
        match self {
            Operation::Eq => buf.push_str(" = "),
            Operation::Gte => buf.push_str(" >= "),
            Operation::Lte => buf.push_str(" <= "),
            Operation::Gt => buf.push_str(" > "),
            Operation::Lt => buf.push_str(" < "),
        }
    }
}
