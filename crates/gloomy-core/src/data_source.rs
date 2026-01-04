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
/// Sort direction for DataGrid columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    /// Ascending order (A-Z, 0-9)
    Ascending,
    /// Descending order (Z-A, 9-0)
    Descending,
}

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

    /// Returns a version identifier that changes whenever data is modified.
    /// This allows the UI to skip expensive updates if the data hasn't changed.
    fn version(&self) -> u64;

    /// Gets the cell value as a formatted string.
    fn cell_text(&self, row: usize, col: usize) -> String;

    /// Gets the cell value for sorting and comparison.
    fn cell_value(&self, row: usize, col: usize) -> CellValue {
        CellValue::Text(self.cell_text(row, col))
    }

    /// Sorts the data by the specified column.
    fn sort(&mut self, col: usize, direction: SortDirection) {
        // Default implementation does nothing
    }

    /// Sets a cell value. Returns true if successful.
    fn set_cell(&mut self, _row: usize, _col: usize, _value: CellValue) -> bool {
        false
    }

    /// Adds a new empty row at end. Returns new row index if supported.
    fn add_row_default(&mut self) -> Option<usize> {
        None
    }

    /// Deletes a row at the given index. Returns true if successful.
    fn delete_row(&mut self, _row: usize) -> bool {
        false
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


/// Simple vector-based data source (Columnar Storage).
///
/// Stores data as a vector of columns (Column-Major), which is more efficient
/// for bulk updates and certain access patterns. The public API generic inputs
/// are transposed on creation.
pub struct VecDataSource {
    headers: Vec<String>,
    // Outer vector is columns, Inner vector is rows.
    // data[col][row]
    data: Vec<Vec<CellValue>>,
    version: u64,
}

impl VecDataSource {
    /// Creates a new VecDataSource.
    ///
    /// # Arguments
    /// * `columns` - Column headers
    /// * `rows` - Row data (Row-Major input is transposed to Column-Major storage)
    pub fn new(columns: Vec<String>, rows: Vec<Vec<CellValue>>) -> Self {
        let col_count = columns.len();
        let row_count = rows.len();
        
        let mut data = vec![Vec::with_capacity(row_count); col_count];
        
        // Transpose row-major input to column-major storage
        for row in rows {
            let len = row.len();
            for (c, val) in row.into_iter().enumerate() {
                if c < col_count {
                    data[c].push(val);
                }
            }
            // Fill missing columns with None if row was too short
            for col_data in data.iter_mut().take(col_count).skip(len) {
                col_data.push(CellValue::None);
            }
        }

        Self { 
            headers: columns, 
            data,
            version: 0 
        }
    }

    /// Creates an empty data source with column headers.
    pub fn with_columns(columns: Vec<String>) -> Self {
        let col_count = columns.len();
        Self {
            headers: columns,
            data: vec![Vec::new(); col_count],
            version: 0,
        }
    }

    /// Adds a row to the data source.
    pub fn add_row(&mut self, row: Vec<CellValue>) {
        let len = row.len();
        for (c, val) in row.into_iter().enumerate() {
            if c < self.data.len() {
                self.data[c].push(val);
            }
        }
        // Fill remaining columns if row is short
        for col_data in self.data.iter_mut().skip(len) {
            col_data.push(CellValue::None);
        }
        self.version += 1;
    }

    /// Gets a reference to the column headers.
    pub fn columns(&self) -> &[String] {
        &self.headers
    }
}

impl DataSource for VecDataSource {
    fn row_count(&self) -> usize {
        if self.data.is_empty() { 0 } else { self.data[0].len() }
    }

    fn column_count(&self) -> usize {
        self.headers.len()
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn cell_text(&self, row: usize, col: usize) -> String {
        self.data
            .get(col)
            .and_then(|c| c.get(row))
            .map(|v| v.to_string())
            .unwrap_or_default()
    }

    fn cell_value(&self, row: usize, col: usize) -> CellValue {
        self.data
            .get(col)
            .and_then(|c| c.get(row))
            .cloned()
            .unwrap_or(CellValue::None)
    }

    fn sort(&mut self, col: usize, direction: SortDirection) {
        if col >= self.data.len() { return; }
        
        let row_count = self.row_count();
        if row_count == 0 { return; }

        // Create indices
        let mut indices: Vec<usize> = (0..row_count).collect();

        // Sort indices based on the specific column
        let target_col = &self.data[col];
        indices.sort_by(|&a, &b| {
            let val_a = &target_col[a];
            let val_b = &target_col[b];
            let cmp = val_a.partial_cmp(val_b).unwrap_or(std::cmp::Ordering::Equal);
            match direction {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });

        // Reorder ALL columns based on new indices
        for col_data in self.data.iter_mut() {
            let mut new_col = Vec::with_capacity(row_count);
            for &idx in &indices {
                new_col.push(col_data[idx].clone());
            }
            *col_data = new_col;
        }
        self.version += 1;
    }

    fn set_cell(&mut self, row: usize, col: usize, value: CellValue) -> bool {
        if let Some(c) = self.data.get_mut(col) {
            if row < c.len() {
                c[row] = value;
                self.version += 1;
                return true;
            }
        }
        false
    }

    fn add_row_default(&mut self) -> Option<usize> {
        for col_data in self.data.iter_mut() {
            col_data.push(CellValue::None);
        }
        self.version += 1;
        Some(self.row_count() - 1)
    }

    fn delete_row(&mut self, row: usize) -> bool {
        if row < self.row_count() {
            for col_data in self.data.iter_mut() {
                col_data.remove(row);
            }
            self.version += 1;
            true
        } else {
            false
        }
    }
}

/// Trait for looking up data sources by ID.
pub trait DataProvider: Send + Sync {
    /// Gets a data source by ID.
    fn get_source(&self, id: &str) -> Option<&dyn DataSource>;
    /// Gets a mutable data source by ID.
    fn get_source_mut(&mut self, id: &str) -> Option<&mut (dyn DataSource + 'static)>;
}

/// Simple HashMap-based data provider.
pub struct MapDataProvider {
    sources: std::collections::HashMap<String, Box<dyn DataSource>>,
}

impl MapDataProvider {
    /// Creates a new empty MapDataProvider.
    pub fn new() -> Self {
        Self {
            sources: std::collections::HashMap::new(),
        }
    }
    
    /// Registers a data source with an ID.
    pub fn register<S: DataSource + 'static>(&mut self, id: impl Into<String>, source: S) {
        self.sources.insert(id.into(), Box::new(source));
    }
}

impl DataProvider for MapDataProvider {
    fn get_source(&self, id: &str) -> Option<&dyn DataSource> {
        self.sources.get(id).map(|b| b.as_ref())
    }

    fn get_source_mut(&mut self, id: &str) -> Option<&mut (dyn DataSource + 'static)> {
        self.sources.get_mut(id).map(move |b| b.as_mut())
    }
}

impl Default for MapDataProvider {
    fn default() -> Self {
        Self::new()
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
