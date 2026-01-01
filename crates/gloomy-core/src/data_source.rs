//! Data source trait for providing data to widgets like DataGrid.
//!
//! This module defines the trait that data providers must implement
//! to work with data-driven widgets.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Trait for providing tabular data to widgets.
///
/// Implement this trait to create custom data sources for the
/// DataGrid widget. The trait allows for efficient data access
/// and supports sorting.
pub trait DataSource: Send + Sync {
    /// Returns the total number of rows in the data source.
    fn row_count(&self) -> usize;

    /// Returns the total number of columns in the data source.
    fn column_count(&self) -> usize;

    /// Gets the cell value as a formatted string.
    ///
    /// # Arguments
    /// * `row` - Zero-based row index
    /// * `col` - Zero-based column index
    fn cell_text(&self, row: usize, col: usize) -> String;

    /// Gets the cell value for sorting and comparison.
    ///
    /// Default implementation returns Text variant of cell_text.
    fn cell_value(&self, row: usize, col: usize) -> CellValue {
        CellValue::Text(self.cell_text(row, col))
    }
}

/// Represents a cell value with type information for sorting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    /// Text/string value
    Text(String),
    /// Floating point number
    Number(f64),
    /// Integer value
    Integer(i64),
    /// Boolean value
    Boolean(bool),
    /// Empty/null value
    None,
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellValue::Text(s) => write!(f, "{}", s),
            CellValue::Number(n) => write!(f, "{}", n),
            CellValue::Integer(i) => write!(f, "{}", i),
            CellValue::Boolean(b) => write!(f, "{}", b),
            CellValue::None => write!(f, ""),
        }
    }
}

impl PartialOrd for CellValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        use CellValue::*;

        match (self, other) {
            (Text(a), Text(b)) => a.partial_cmp(b),
            (Number(a), Number(b)) => a.partial_cmp(b),
            (Integer(a), Integer(b)) => a.partial_cmp(b),
            (Boolean(a), Boolean(b)) => a.partial_cmp(b),
            (None, None) => Some(Ordering::Equal),
            (None, _) => Some(Ordering::Less),
            (_, None) => Some(Ordering::Greater),
            // Different types: compare as strings
            (a, b) => a.to_string().partial_cmp(&b.to_string()),
        }
    }
}

/// Simple vector-based data source.
///
/// Stores data as a vector of rows, where each row is a vector
/// of CellValues. Suitable for small to medium datasets.
pub struct VecDataSource {
    columns: Vec<String>,
    rows: Vec<Vec<CellValue>>,
}

impl VecDataSource {
    /// Creates a new VecDataSource.
    ///
    /// # Arguments
    /// * `columns` - Column headers
    /// * `rows` - Row data (each row must have same length as columns)
    pub fn new(columns: Vec<String>, rows: Vec<Vec<CellValue>>) -> Self {
        Self { columns, rows }
    }

    /// Creates an empty data source with column headers.
    pub fn with_columns(columns: Vec<String>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
        }
    }

    /// Adds a row to the data source.
    pub fn add_row(&mut self, row: Vec<CellValue>) {
        self.rows.push(row);
    }

    /// Gets a reference to the column headers.
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Gets a mutable reference to the rows for in-place sorting.
    pub fn rows_mut(&mut self) -> &mut Vec<Vec<CellValue>> {
        &mut self.rows
    }
}

impl DataSource for VecDataSource {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn column_count(&self) -> usize {
        self.columns.len()
    }

    fn cell_text(&self, row: usize, col: usize) -> String {
        self.rows
            .get(row)
            .and_then(|r| r.get(col))
            .map(|v| v.to_string())
            .unwrap_or_default()
    }

    fn cell_value(&self, row: usize, col: usize) -> CellValue {
        self.rows
            .get(row)
            .and_then(|r| r.get(col))
            .cloned()
            .unwrap_or(CellValue::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_value_ordering() {
        assert!(CellValue::Integer(1) < CellValue::Integer(2));
        assert!(CellValue::Number(1.5) < CellValue::Number(2.5));
        assert!(CellValue::Text("a".to_string()) 
            < CellValue::Text("b".to_string()));
        assert!(CellValue::Boolean(false) < CellValue::Boolean(true));
        assert!(CellValue::None < CellValue::Integer(1));
    }

    #[test]
    fn test_vec_data_source() {
        let mut ds = VecDataSource::with_columns(
            vec!["Name".to_string(), "Age".to_string()]
        );
        ds.add_row(vec![
            CellValue::Text("Alice".to_string()),
            CellValue::Integer(30),
        ]);
        ds.add_row(vec![
            CellValue::Text("Bob".to_string()),
            CellValue::Integer(25),
        ]);

        assert_eq!(ds.row_count(), 2);
        assert_eq!(ds.column_count(), 2);
        assert_eq!(ds.cell_text(0, 0), "Alice");
        assert_eq!(ds.cell_text(1, 1), "25");
    }
}
