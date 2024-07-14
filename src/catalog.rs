use crate::picture_entry::PictureEntry;
use std::cmp::Ordering::Equal;
use crate::order::Order;

#[derive(Debug)]
pub struct Catalog {
    picture_entries: Vec<PictureEntry>,
    index: usize,
    page_size: usize,
}

impl Catalog {

    pub fn new() -> Self {
        Catalog {
            picture_entries : Vec::new(),
            index: 0,
            page_size: 1,
        }
    }

    pub fn length(&self) -> usize {
        self.picture_entries.len()
    }

    pub fn add_picture_entries(&mut self, picture_entries: &mut Vec<PictureEntry>) {
        self.picture_entries.append(picture_entries)
    }

    pub fn set_page_size(&mut self, page_size: usize) {
        assert!(page_size > 0 && page_size <= 10);
        self.page_size = page_size;
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn page_length(&self) -> usize {
        self.page_size * self.page_size
    }

    pub fn page_index_of(&self, index: usize) -> usize {
        index - (index % self.page_length())
    }
    pub fn page_index(&self) -> usize {
        self.page_index_of(self.index)
    }

    pub fn sort_by(&mut self, order: Order) {
        match order {
            Order::Colors => self.picture_entries.sort_by(|a, b| { a.colors.cmp(&b.colors) }),
            Order::Date => self.picture_entries.sort_by(|a, b| { a.modified_time.cmp(&b.modified_time) }),
            Order::Name => self.picture_entries.sort_by(|a, b| { a.original_file_path().cmp(&b.original_file_path()) }),
            Order::Size => self.picture_entries.sort_by(|a, b| { a.file_size.cmp(&b.file_size)} ),
            Order::Value => self.picture_entries.sort_by(|a, b|  { a.cmp_rank(&b) }),
            Order::Label => self.picture_entries.sort_by(|a, b| { a.cmp_label(&b) }),
            Order::Palette => self.picture_entries.sort_by(|a, b| { a.palette.cmp(&b.palette) }),
        }
    }

    pub fn can_move_to_index(&self, index: usize) -> bool {
        index < self.picture_entries.len()
    }

    pub fn move_to_index(&mut self, index: usize) {
        self.index = index
    }

    pub fn move_next_page(&mut self) {
        let new_index = self.page_index() + self.page_length();
        self.index = if new_index < self.length() {
            new_index
        } else {
            0
        };
    }

    pub fn move_prev_page(&mut self) {
        self.index = if self.page_index() >= self.page_length() {
            self.page_index() - self.page_length()
        } else {
            self.page_index_of(self.length()-1)
        }
    }

    pub fn current(&self) -> Option<PictureEntry> {
        if self.index < self.picture_entries.len() {
            Some(self.picture_entries[self.index].clone())
        } else {
            None
        }
    }

    pub fn set_current_label(&mut self, label: String) {
        if self.index < self.picture_entries.len() {
            self.picture_entries[self.index].set_label(label)
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
            make_picture_entry(String::from("photos/foo.jpeg"), 100, 5, day_d, Rank::NoStar, None, Some(String::from("foo"))),
            make_picture_entry(String::from("photos/bar.jpeg"), 1000, 15, day_b, Rank::ThreeStars, None, None),
            make_picture_entry(String::from("photos/qux.jpeg"), 10, 25, day_c, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None),
            make_picture_entry(String::from("photos/bub.jpeg"), 100, 25, day_a, Rank::OneStar, None, Some(String::from("xanadoo"))))
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
        { catalog_rc.borrow_mut().sort_by(Order::Label) };
        { catalog_rc.borrow_mut().move_to_index(0) };
        { assert_eq!(String::from("foo.jpeg"),
            catalog_rc.borrow().current().unwrap().original_file_name()) };
        { catalog_rc.borrow_mut().move_to_index(1) };
        { assert_eq!(String::from("bub.jpeg"),
            catalog_rc.borrow().current().unwrap().original_file_name()) };
        { catalog_rc.borrow_mut().sort_by(Order::Palette) };
        { catalog_rc.borrow_mut().move_to_index(3) };
        { assert_eq!(String::from("qux.jpeg"),
            catalog_rc.borrow().current().unwrap().original_file_name()) };
    }

    #[test]
    fn page_index_depends_on_page_size_and_index() {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let mut example = my_entries();
        let mut other_entries = vec![
            make_picture_entry(String::from("photos/joe.jpeg"), 100, 5, day_a, Rank::NoStar, None, Some(String::from("foo"))),
            make_picture_entry(String::from("photos/gus.jpeg"), 1000, 15, day_a, Rank::ThreeStars, None, None),
            make_picture_entry(String::from("photos/zoo.jpeg"), 10, 25, day_a, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None)];
        let catalog_rc = Rc::new(RefCell::new(Catalog::new()));
        { catalog_rc.borrow_mut().add_picture_entries(&mut example) };
        { catalog_rc.borrow_mut().add_picture_entries(&mut other_entries) };
        { catalog_rc.borrow_mut().set_page_size(2) };
        { assert_eq!(4, catalog_rc.borrow().page_length()) };
        { catalog_rc.borrow_mut().move_to_index(0) };
        { assert_eq!(0, catalog_rc.borrow().page_index()) };
        { catalog_rc.borrow_mut().move_to_index(6) };
        { assert_eq!(4, catalog_rc.borrow().page_index()) };
    }

    #[test]
    fn after_moving_next_page_index_depends_on_page_size() {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let mut example = my_entries();
        let mut other_entries = vec![
            make_picture_entry(String::from("photos/joe.jpeg"), 100, 5, day_a, Rank::NoStar, None, Some(String::from("foo"))),
            make_picture_entry(String::from("photos/gus.jpeg"), 1000, 15, day_a, Rank::ThreeStars, None, None),
            make_picture_entry(String::from("photos/zoo.jpeg"), 10, 25, day_a, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None)];
        let catalog_rc = Rc::new(RefCell::new(Catalog::new()));
        { catalog_rc.borrow_mut().add_picture_entries(&mut example) };
        { catalog_rc.borrow_mut().add_picture_entries(&mut other_entries) };
        { catalog_rc.borrow_mut().set_page_size(2) };
        { catalog_rc.borrow_mut().move_to_index(2) };
        { catalog_rc.borrow_mut().move_next_page() };
        { assert_eq!(4, catalog_rc.borrow().page_index()) };
        { catalog_rc.borrow_mut().move_next_page() };
        { assert_eq!(0, catalog_rc.borrow().page_index()) };
        { catalog_rc.borrow_mut().move_prev_page() };
        { assert_eq!(4, catalog_rc.borrow().page_index()) };
    }

}
