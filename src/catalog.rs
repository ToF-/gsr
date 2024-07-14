use crate::picture_entry::PictureEntry;
use std::cmp::Ordering::Equal;
use crate::order::Order;

pub struct Catalog {
    picture_entries: Vec<PictureEntry>,
    index: usize,
}

impl Catalog {

    pub fn new() -> Self {
        Catalog {
            picture_entries : Vec::new(),
            index: 0,
        }
    }

    pub fn length(&self) -> usize {
        self.picture_entries.len()
    }

    pub fn add_picture_entries(&mut self, picture_entries: &mut Vec<PictureEntry>) {
        self.picture_entries.append(picture_entries)
    }

    pub fn sort_by(&mut self, order: Order) {
        match order {
            Order::Colors => self.picture_entries.sort_by(|a, b| { a.colors.cmp(&b.colors) }),
            Order::Date => self.picture_entries.sort_by(|a, b| { a.modified_time.cmp(&b.modified_time) }),
            Order::Name => self.picture_entries.sort_by(|a, b| { a.original_file_path().cmp(&b.original_file_path()) }),
            Order::Size => self.picture_entries.sort_by(|a, b| { a.file_size.cmp(&b.file_size)} ),
            Order::Value => self.picture_entries.sort_by(|a, b|  {
                let cmp = (a.rank.clone() as usize).cmp(&(b.rank.clone() as usize));
                if cmp == Equal {
                    a.original_file_path().cmp(&b.original_file_path())
                } else {
                    cmp
                }
            }),
        }
        
    }

    pub fn can_move_to_index(&self, index: usize) -> bool {
        index < self.picture_entries.len()
    }

    pub fn move_to_index(&mut self, index: usize) {
        self.index = index
    }

    pub fn current(&self) -> Option<PictureEntry> {
        if self.index < self.picture_entries.len() {
            Some(self.picture_entries[self.index].clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::rank::Rank;
    use crate::picture_entry::make_picture_entry;
    use std::time::SystemTime;
    use chrono::DateTime;

    fn my_entries() -> Vec<PictureEntry> {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let day_b: SystemTime = DateTime::parse_from_rfc2822("Sat, 1 Jul 2023 10:52:37 GMT").unwrap().into();
        let day_c: SystemTime = DateTime::parse_from_rfc2822("Mon, 1 Jan 2024 10:52:37 GMT").unwrap().into();
        let day_d: SystemTime = DateTime::parse_from_rfc2822("Mon, 1 Jan 2024 11:52:37 GMT").unwrap().into();
        vec!(
            make_picture_entry(String::from("photos/foo.jpeg"), 100, 5, day_d, Rank::NoStar),
            make_picture_entry(String::from("photos/bar.jpeg"), 1000, 15, day_b, Rank::ThreeStars),
            make_picture_entry(String::from("photos/qux.jpeg"), 10, 25, day_c, Rank::TwoStars),
            make_picture_entry(String::from("photos/bub.jpeg"), 100, 25, day_a, Rank::OneStar))
    }

    #[test]
    fn at_creation_length_is_0() {
        let catalog = Catalog::new();
        assert_eq!(catalog.length(), 0);
        assert_eq!(true, catalog.current().is_none());
    }

    #[test]
    fn after_adding_entries_length_is_updated() {
        let mut catalog = Catalog::new();
        let mut example = my_entries();
        catalog.add_picture_entries(&mut example);
        assert_eq!(catalog.length(), 4);
    }

    #[test]
    fn cannot_move_beyond_length() {
        let mut catalog = Catalog::new();
        let mut example = my_entries();
        catalog.add_picture_entries(&mut example);
        assert_eq!(false, catalog.can_move_to_index(4));
    }

    #[test]
    fn sorting_catalog_by_different_criteria() {
        let mut example = my_entries();
        let catalog_rc = Rc::new(RefCell::new(Catalog::new()));
        { catalog_rc.borrow_mut().add_picture_entries(&mut example) };
        { catalog_rc.borrow_mut().sort_by(Order::Size) };
        { catalog_rc.borrow_mut().move_to_index(0) };
        { assert_eq!(String::from("qux.jpeg"),
            catalog_rc.borrow().current().unwrap().original_file_name()) };
        { catalog_rc.borrow_mut().sort_by(Order::Date) };
        { catalog_rc.borrow_mut().move_to_index(0) };
        { assert_eq!(String::from("bub.jpeg"),
            catalog_rc.borrow().current().unwrap().original_file_name()) };
        { catalog_rc.borrow_mut().sort_by(Order::Name) };
        { catalog_rc.borrow_mut().move_to_index(0) };
        { assert_eq!(String::from("bar.jpeg"),
            catalog_rc.borrow().current().unwrap().original_file_name()) };
        { catalog_rc.borrow_mut().sort_by(Order::Colors) };
        { catalog_rc.borrow_mut().move_to_index(0) };
        { assert_eq!(String::from("foo.jpeg"),
            catalog_rc.borrow().current().unwrap().original_file_name()) };
        { catalog_rc.borrow_mut().sort_by(Order::Value) };
        { catalog_rc.borrow_mut().move_to_index(3) };
        { assert_eq!(String::from("foo.jpeg"),
            catalog_rc.borrow().current().unwrap().original_file_name()) };
    }
}
