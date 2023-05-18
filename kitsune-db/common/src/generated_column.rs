use sea_orm::{
    sea_query::{IntoIden, SimpleExpr},
    DatabaseBackend, DynIden, Iden,
};
use std::fmt;

/// Constructor for a stored generated column
pub struct StoredGeneratedColumn {
    col_type: DynIden,
    generate_expr: Option<SimpleExpr>,
}

impl StoredGeneratedColumn {
    /// Create a new `StoredGeneratedColumn` type
    pub fn new<T>(col_type: T) -> Self
    where
        T: IntoIden,
    {
        Self {
            col_type: col_type.into_iden(),
            generate_expr: None,
        }
    }

    /// Expression that generates the value returned by this function
    #[must_use]
    pub fn generate_expr<T>(self, generate_expr: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        Self {
            generate_expr: Some(generate_expr.into()),
            ..self
        }
    }
}

impl Iden for StoredGeneratedColumn {
    fn unquoted(&self, s: &mut dyn fmt::Write) {
        self.col_type.unquoted(s);
        write!(s, " GENERATED ALWAYS AS (").unwrap();

        if let Some(ref generate_expr) = self.generate_expr {
            let mut buf = String::new();
            DatabaseBackend::Postgres
                .get_query_builder()
                .prepare_simple_expr(generate_expr, &mut buf);

            s.write_str(&buf).unwrap();
        }

        write!(s, ") STORED").unwrap();
    }
}
