use crate::catalog::Coords;
use crate::catalog::Catalog;

#[derive(Debug,Clone)]
pub struct Navigator {
    pub index: usize,
    pub last_index: Option<usize>,
    pub len: usize,
    pub new_page_size: Option<usize>,
    pub page_changed: bool,
    pub page_size: usize,
}

impl Navigator {

    pub fn index_from_position(self, coords: Coords) -> Option<usize> {
        let index = (self.page_index() + coords.0 as usize + coords.1 as usize * self.page_size()) as usize;
        if index < self.len {
            Some(index)
        } else {
            None
        }
    }
    pub fn position_from_index(&self, index: usize) -> Coords {
        let start_index = self.page_index_of(index);
        let offset = index - start_index;
        let row = offset / self.cells_per_row();
        let col = offset % self.cells_per_row();
        (col, row)
    }

    pub fn cells_per_row(&self) -> usize {
        self.page_size()
    }
    pub fn page_index_of(&self, index: usize) -> usize {
        index - (index % self.page_length())
    }

    pub fn page_index(&self) -> usize {
        self.page_index_of(self.index)
    }

    pub fn page_changed(&self) -> bool {
        self.page_changed
    }

    pub fn change_page(&mut self) {
        self.page_changed = true
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn page_length(&self) -> usize {
        self.page_size * self.page_size
    }

    pub fn index(&self) -> Option<usize> {
        if self.index < self.len {
            Some(self.index)
        } else {
            None
        }

    }

}


