use crate::picture_entry::PictureEntry;
use crate::order::Order;
use crate::direction::Direction;

#[derive(Debug)]
pub struct Catalog {
    picture_entries: Vec<PictureEntry>,
    index: usize,
    page_size: usize,
    page_limit_on: bool,
    input: Option<String>,
}

impl Catalog {

    pub fn new() -> Self {
        Catalog {
            picture_entries : Vec::new(),
            index: 0,
            page_size: 1,
            page_limit_on: true,
            input: None,
        }
    }

    pub fn length(&self) -> usize {
        self.picture_entries.len()
    }

    pub fn last(&self) -> usize {
        self.length()-1
    }

    pub fn index(&self) -> usize {
        self.index
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

    pub fn page_limit_on(&self) -> bool {
        self.page_limit_on
    }

    pub fn toggle_page_limit(&mut self) {
        self.page_limit_on = !self.page_limit_on
    }

    pub fn page_index_of(&self, index: usize) -> usize {
        index - (index % self.page_length())
    }
    pub fn page_index(&self) -> usize {
        self.page_index_of(self.index)
    }

    pub fn input_on(&self) -> bool {
        self.input.is_some()
    }

    pub fn cancel_input(&mut self) {
        self.input = None
    }

    pub fn input(&self) -> String {
        self.input.clone().unwrap()
    }

    pub fn begin_input(&mut self) {
        self.input = Some(String::from(""))
    }

    pub fn add_input_char(&mut self, ch: char) {
        self.input = self.input.clone().map( |s| {
            let mut t = s.clone();
            t.push(ch);
            t
        })
    }

    pub fn del_input_char(&mut self) {
        self.input = self.input.clone().map( |s| {
            let mut t = s.clone();
            t.pop();
            t
        })
    }

    pub fn find_input_pattern(&mut self) -> bool {
        let result = if let Some(pattern) = &self.input {
            if let Some(index) = self.picture_entries.iter().position(|entry|
                entry.original_file_name().contains(&*pattern)) {
                self.move_to_index(index);
                true
            } else {
                false
            }
        } else {
            false
        };
        self.input = None;
        result
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

    pub fn can_move_towards(&self, direction: Direction) -> bool {
        ! self.page_limit_on ||
            match direction {
                Direction::Left => self.page_size > 1 && self.index % self.page_size > 0,
                Direction::Right => self.page_size > 1 && (self.index+1) % self.page_size > 0,
                Direction::Up => self.page_size > 1 && self.index >= self.page_size,
                Direction::Down => self.page_size > 1 && (self.index + self.page_size) < self.page_length(),
            }
    }

    pub fn move_towards(&mut self, direction: Direction) {
        if self.can_move_towards(direction.clone()) {
            match direction {
                Direction::Right => {
                    if self.index + 1 < self.length() {
                        self.index += 1
                    }
                },
                Direction::Left => {
                    if self.index > 0 {
                        self.index -= 1
                    }
                },
                Direction::Down => {
                    if self.index + self.page_size < self.length() {
                        self.index += self.page_size
                    } else {
                        self.index = 0
                    }
                },
                Direction::Up => {
                    if self.index >= self.page_size {
                        self.index -= self.page_size
                    } else {
                        let offset = self.index - self.page_index();
                        let new_page_index = self.last() - (self.last() % self.page_length());
                        let new_index = new_page_index + self.page_length() - self.page_size() + offset;
                        self.index = if new_index < self.length() {
                            new_index
                        } else {
                            self.last()
                        }
                    }
                },
            }
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let mut catalog = Catalog::new();
        catalog.add_picture_entries(&mut example);
        catalog.sort_by(Order::Size);
        catalog.move_to_index(0);
        assert_eq!(String::from("qux.jpeg"),
            catalog.current().unwrap().original_file_name());
        catalog.sort_by(Order::Date);
        catalog.move_to_index(0);
        assert_eq!(String::from("bub.jpeg"),
            catalog.current().unwrap().original_file_name());
        catalog.sort_by(Order::Name);
        catalog.move_to_index(0);
        assert_eq!(String::from("bar.jpeg"),
            catalog.current().unwrap().original_file_name());
        catalog.sort_by(Order::Colors);
        catalog.move_to_index(0);
        assert_eq!(String::from("foo.jpeg"),
            catalog.current().unwrap().original_file_name());
        catalog.sort_by(Order::Value);
        catalog.move_to_index(3);
        assert_eq!(String::from("foo.jpeg"),
            catalog.current().unwrap().original_file_name());
        catalog.sort_by(Order::Label);
        catalog.move_to_index(0);
        assert_eq!(String::from("foo.jpeg"),
            catalog.current().unwrap().original_file_name());
        catalog.move_to_index(1);
        assert_eq!(String::from("bub.jpeg"),
            catalog.current().unwrap().original_file_name());
        catalog.sort_by(Order::Palette);
        catalog.move_to_index(3);
        assert_eq!(String::from("qux.jpeg"),
            catalog.current().unwrap().original_file_name());
    }

    #[test]
    fn page_index_depends_on_page_size_and_index() {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let mut example = my_entries();
        let mut other_entries = vec![
            make_picture_entry(String::from("photos/joe.jpeg"), 100, 5, day_a, Rank::NoStar, None, Some(String::from("foo"))),
            make_picture_entry(String::from("photos/gus.jpeg"), 1000, 15, day_a, Rank::ThreeStars, None, None),
            make_picture_entry(String::from("photos/zoo.jpeg"), 10, 25, day_a, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None)];
        let mut catalog = Catalog::new();
        catalog.add_picture_entries(&mut example);
        catalog.add_picture_entries(&mut other_entries);
        catalog.set_page_size(2);
        assert_eq!(4, catalog.page_length());
        catalog.move_to_index(0);
        assert_eq!(0, catalog.page_index());
        catalog.move_to_index(6);
        assert_eq!(4, catalog.page_index());
    }

    #[test]
    fn after_moving_next_page_index_depends_on_page_size() {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let mut example = my_entries();
        let mut other_entries = vec![
            make_picture_entry(String::from("photos/joe.jpeg"), 100, 5, day_a, Rank::NoStar, None, Some(String::from("foo"))),
            make_picture_entry(String::from("photos/gus.jpeg"), 1000, 15, day_a, Rank::ThreeStars, None, None),
            make_picture_entry(String::from("photos/zoo.jpeg"), 10, 25, day_a, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None)];
        let mut catalog = Catalog::new();
        catalog.add_picture_entries(&mut example);
        catalog.add_picture_entries(&mut other_entries);
        catalog.set_page_size(2);
        assert_eq!(2, catalog.page_size());
        catalog.move_to_index(2);
        catalog.move_next_page();
        assert_eq!(4, catalog.page_index());
        catalog.move_next_page();
        assert_eq!(0, catalog.page_index());
        catalog.move_prev_page();
        assert_eq!(4, catalog.page_index());
    }

    #[test]
    fn moving_next_picture_can_be_blocked_or_allowed() {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let mut example = my_entries();
        let mut other_entries = vec![
            make_picture_entry(String::from("photos/joe.jpeg"), 100, 5, day_a, Rank::NoStar, None, Some(String::from("foo"))),
            make_picture_entry(String::from("photos/gus.jpeg"), 1000, 15, day_a, Rank::ThreeStars, None, None),
            make_picture_entry(String::from("photos/zoo.jpeg"), 10, 25, day_a, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None)];
        let mut catalog = Catalog::new();
        catalog.add_picture_entries(&mut example);
        catalog.add_picture_entries(&mut other_entries);
        assert_eq!(7, catalog.length());
        assert_eq!(true, catalog.page_limit_on());
        catalog.set_page_size(2);
        catalog.move_to_index(0);
        catalog.move_towards(Direction::Right);
        assert_eq!(1, catalog.index());
        catalog.move_towards(Direction::Down);
        assert_eq!(3, catalog.index());
        catalog.move_towards(Direction::Left);
        assert_eq!(2, catalog.index());
        assert_eq!(true, catalog.can_move_towards(Direction::Up));
        catalog.move_towards(Direction::Up);
        assert_eq!(0, catalog.index());
        assert_eq!(false, catalog.can_move_towards(Direction::Left));
        assert_eq!(false, catalog.can_move_towards(Direction::Up));
        assert_eq!(true, catalog.can_move_towards(Direction::Right));
        assert_eq!(true, catalog.can_move_towards(Direction::Down));
        catalog.move_towards(Direction::Right);
        assert_eq!(false, catalog.can_move_towards(Direction::Right));
        catalog.move_towards(Direction::Down);
        assert_eq!(false, catalog.can_move_towards(Direction::Down));
        catalog.toggle_page_limit();
        assert_eq!(false, catalog.page_limit_on());
        assert_eq!(3, catalog.index());
        catalog.move_towards(Direction::Down);
        assert_eq!(5, catalog.index());
        assert_eq!(true, catalog.can_move_towards(Direction::Down));
        catalog.move_towards(Direction::Down);
        assert_eq!(0, catalog.index());
        catalog.move_towards(Direction::Right);
        assert_eq!(1, catalog.index());
        assert_eq!(true, catalog.can_move_towards(Direction::Up));
        catalog.move_towards(Direction::Up);
        assert_eq!(6, catalog.index()); // because there's no picture entry in pos 7
        catalog.move_to_index(5);
        assert_eq!(true, catalog.can_move_towards(Direction::Down));
        catalog.move_towards(Direction::Down);
        assert_eq!(0, catalog.index()); // because there's no picture entry in pos 7
    }

    #[test]
    fn editing_input() {
        let mut catalog = Catalog::new();
        assert_eq!(false, catalog.input_on());
        catalog.begin_input();
        assert_eq!(true, catalog.input_on());
        catalog.add_input_char('F');
        catalog.add_input_char('o');
        catalog.add_input_char('o');
        catalog.add_input_char('-');
        assert_eq!(String::from("Foo-"), catalog.input());
        catalog.del_input_char();
        assert_eq!(String::from("Foo"), catalog.input());
        catalog.cancel_input();
        assert_eq!(false, catalog.input_on());
    }

    #[test]
    fn finding_a_picture_entry_by_input_pattern() {
        let mut example = my_entries();
        let mut catalog = Catalog::new();
        catalog.add_picture_entries(&mut example);
        catalog.sort_by(Order::Size);
        catalog.move_to_index(0);
        assert_eq!(String::from("qux.jpeg"),catalog.current().unwrap().original_file_name());
        catalog.begin_input();
        catalog.add_input_char('f');
        catalog.add_input_char('o');
        assert_eq!(true, catalog.find_input_pattern());
        assert_eq!(String::from("foo.jpeg"), catalog.current().unwrap().original_file_name());
        catalog.begin_input();
        catalog.add_input_char('q');
        catalog.add_input_char('a');
        assert_eq!(false, catalog.find_input_pattern());
    }
}
