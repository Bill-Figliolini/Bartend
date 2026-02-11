use std::fmt::Write;

/// Struct for holding the name of a DB table and its columns.
/// 
/// Precondition:
/// At least one column must be inserted before any reading function is called.
#[derive(Debug)]
pub(super) struct Schema {
    name: String,
    columns: Vec<String>,
}

impl Schema {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: vec![],
        }
    }
    pub fn column(mut self, column: &str) -> Self {
        self.columns.push(column.to_string());
        self
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub const fn columns(&self) -> &Vec<String> {
        assert!(!self.columns.is_empty());
        &self.columns
    }
    pub fn columns_string(&self) -> String {
        assert!(!self.columns.is_empty());
        let mut columns = self.columns[0].clone();
        for column in self.columns.iter().skip(1) {
            _ = write!(columns, ", {column}");
        }
        columns
    }

    pub fn get_autoinsert_statement(&self) -> String {
        assert!(!self.columns.is_empty());
        let mut clause = format!("{} (", self.name);
        for column in self.columns.iter().skip(1) {
            if !clause.ends_with('(') {
                clause.push_str(", ");
            }

            _ = write!(clause, "{column}");
        }

        clause.push(')');
        clause
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod preconditions {
        use super::*;

        mod schema_must_have_one_column_to_be_read {
            use super::*;
            #[test]
            #[should_panic]
            fn columns() {
                let schema = Schema::new("name");

                _ = schema.columns()
            }
            #[test]
            #[should_panic]
            fn columns_string() {
                let schema = Schema::new("name");

                _ = schema.columns_string()
            }
            #[test]
            #[should_panic]
            fn get_auto_insert_statement() {
                let schema = Schema::new("name");

                _ = schema.get_autoinsert_statement()
            }
        }
    }
}
