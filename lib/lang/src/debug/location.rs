use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Location {
    line_number: usize,
    column_number: usize,
}

impl Location {
    pub fn new(line_number: usize, column_number: usize) -> Self {
        Self {
            line_number,
            column_number,
        }
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn column_number(&self) -> usize {
        self.column_number
    }

    pub fn increment_line_number(&self) -> Self {
        Self::new(self.line_number + 1, 1)
    }

    pub fn increment_column_number(&self) -> Self {
        Self::new(self.line_number, self.column_number + 1)
    }
}
