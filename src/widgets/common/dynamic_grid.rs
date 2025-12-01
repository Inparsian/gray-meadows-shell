use gtk4::prelude::*;
use relm4::RelmRemoveAllExt;

pub struct DynamicGrid {
    pub grid: gtk4::Grid,
    pub items: Vec<gtk4::Widget>,
    pub rows: usize,
    pub columns: usize,
}

impl DynamicGrid {
    pub fn new(rows: usize) -> Self {
        let grid = gtk4::Grid::new();
        grid.set_row_homogeneous(true);
        grid.set_column_homogeneous(true);
        grid.set_row_spacing(8);
        grid.set_column_spacing(8);

        Self {
            grid,
            items: Vec::new(),
            rows,
            columns: 1  
        }
    }

    pub fn append(&mut self, item: &impl IsA<gtk4::Widget>) {
        self.items.push(item.as_ref().clone());
        self.update_grid();
    }

    pub fn update_grid(&mut self) {
        // Add a column if rows > rows
        if self.items.len() > self.rows * self.columns {
            self.grid.insert_column(self.columns as i32);
            self.columns += 1;
        }

        // Clear the grid
        self.grid.remove_all();

        // Add items to the grid
        for (index, item) in self.items.iter().enumerate() {
            let row = index / self.rows;
            let col = index % self.rows;
            self.grid.attach(item, col as i32, row as i32, 1, 1);
        }
    }
}