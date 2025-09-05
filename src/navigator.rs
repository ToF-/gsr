use crate::catalog::Catalog;

#[derive(Debug,Clone)]
pub struct Navigator {
    pub page_changed: bool,
    pub page_size: usize,
}

impl Navigator {

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

}


