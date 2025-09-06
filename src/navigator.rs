use crate::catalog::Coords;
use crate::catalog::Catalog;

#[derive(Debug,Clone)]
pub struct Navigator {
    done: bool,
    index: usize,
    last_index: Option<usize>,
    start_index: Option<usize>,
    length: usize,
    new_page_size: Option<usize>,
    page_changed: bool,
    page_size: usize,
}

impl Navigator {

    pub fn new() -> Self {
        Navigator {
            done: false,
            index: 0,
            page_size: 1,
            page_changed: false,
            new_page_size: None,
            last_index: None,
            start_index: None,
            length: 0,
        }
    }

    pub fn index_from_position(self, coords: Coords) -> Option<usize> {
        let index = (self.page_index() + coords.0 as usize + coords.1 as usize * self.page_size()) as usize;
        if index < self.length {
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

    pub fn new_page_size(&self) -> Option<usize> {
        self.new_page_size
    }

    pub fn start_index(&self) -> Option<usize> {
        self.start_index
    }
    pub fn set_length(&self, length: usize) -> Self {
        Navigator {
            length: length,
            ..self.clone()
        }
    }
    pub fn set_new_page_size(&self, size: usize) -> Self {
        Navigator {
            new_page_size: Some(size),
            last_index: Some(self.index),
            ..self.clone()
        }
    }

    pub fn move_to_index(&self, index: usize) -> Self {
        if index != self.index {
            let old_page_index = self.page_index();
            Navigator {
                index: index,
                ..self.clone()
            }.change_page_from_old(old_page_index)
        } else {
            self.clone()
        }
    }
    pub fn move_to_last_index(&self) -> Self {
        match self.last_index {
            Some(index) => self.move_to_index(index),
            None => self.move_to_index(0),
        }
    }
    pub fn exit(&self) -> Self {
        Navigator {
            done: true,
            ..self.clone()
        }
    }

    pub fn done(&self) -> bool {
        self.done
    }

    pub fn start_set(&self) -> Self {
        Navigator {
            start_index: Some(self.index),
            ..self.clone()
        }
    }

    pub fn cancel_set(&self) -> Self {
        Navigator {
            start_index: None,
            ..self.clone()
        }
    }

    pub fn set_page_size(&self, page_size: usize) -> Self {
        Navigator {
            page_size: page_size,
            ..self.clone()
        }
    }
    pub fn set_last_index(&self) -> Self {
        Navigator{
            last_index: Some(self.index),
            ..self.clone()
        }
    }
    pub fn length(&self) -> usize {
        self.length
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

    pub fn set_index(&self, index: usize) -> Self {
        Navigator {
            index: index,
            ..self.clone()
        }
    }
    pub fn change_page_from_old(&self, old_page_index:usize) -> Self {
        if self.page_index() != old_page_index {
            Navigator {
                page_changed: true,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn page_length(&self) -> usize {
        self.page_size * self.page_size
    }

    pub fn index(&self) -> Option<usize> {
        if self.index < self.length {
            Some(self.index)
        } else {
            None
        }

    }

}


