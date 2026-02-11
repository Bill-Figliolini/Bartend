use std::fmt::Write;

/// Struct for holding the name of a DB table and its columns.
///
/// Columns is initialized with an id field.
#[derive(Debug)]
pub(super) struct Schema {
    name: String,
    columns: Vec<String>,
}

impl Schema {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: vec!["id".to_string()],
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
        &self.columns
    }
    pub fn columns_string(&self) -> String {
        let mut columns = self.columns[0].clone();
        for column in self.columns.iter().skip(1) {
            _ = write!(columns, ", {column}");
        }
        columns
    }

    pub fn autoinsert(&self) -> String {
        let mut clause = format!("{} ({}", self.name, self.columns[1]);
        for column in self.columns.iter().skip(2) {
            _ = write!(clause, ", {column}");
        }
        clause.push(')');
        clause
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod usage {
        use super::*;

        mod insert {
            use super::*;

            #[test]
            fn provides_valid_sql() {
                let schema = Schema::new("items").column("name").column("quantity");

                assert_eq!(schema.autoinsert(), "items (name, quantity)")
            }
        }
        mod columns {
            use super::*;

            #[test]
            fn provides_string_of_all_columns() {
                let schema = Schema::new("items").column("name").column("quantity");

                assert_eq!(schema.columns_string(), "id, name, quantity")
            }
        }
    }
}
