use crate::catalog::Catalog;

#[derive(Debug,Clone)]
pub struct Navigator {
    page_changed: bool,
}

impl Navigator {
    pub fn new() -> Self {
        Navigator {
            page_changed: false,
        }
    }

    pub fn page_changed(&self) -> bool {
        self.page_changed
    }

    pub fn change_page(&mut self) {
        self.page_changed = true
    }
}


