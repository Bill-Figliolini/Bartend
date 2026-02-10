use std::fmt::Write;

pub(super) struct Schema {
    name: String,
    columns: Vec<String>,
}

pub(super) struct SchemaBuilder {
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
        let mut clause = self.name.clone();
        clause.push_str(" (");
        for column in &self.columns {
            if column == "id" {
                continue;
            }
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
mod test {}
