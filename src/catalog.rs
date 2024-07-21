use regex::Regex;
use std::io::{Result, Error, ErrorKind};
use std::fs::read_to_string;
use crate::picture_entry::PictureEntry;
use crate::path::{get_picture_file_paths, check_file, is_thumbnail};
use std::path::PathBuf;
use crate::order::Order;
use crate::direction::Direction;

#[derive(Debug)]
pub struct Catalog {
    picture_entries: Vec<PictureEntry>,
    index: usize,
    page_size: usize,
    page_limit_on: bool,
    input: Option<String>,
    label: Option<String>,
    palette_on: bool,
    full_size_on: bool,
    start_index: Option<usize>,
}

impl Catalog {

    // creation

    pub fn new() -> Self {
        Catalog {
            picture_entries : Vec::new(),
            index: 0,
            page_size: 1,
            page_limit_on: true,
            input: None,
            label: None,
            palette_on: false,
            full_size_on: false,
            start_index: None,
        }
    }

    pub fn add_picture_entries(&mut self, picture_entries: &mut Vec<PictureEntry>) {
        self.picture_entries.append(picture_entries)
    }

    pub fn add_picture_entries_from_dir(&mut self, directory: &str, pattern_opt: Option<String>) -> Result<()> {
        match get_picture_file_paths(directory) {
            Ok(file_paths) => {
                for file_path in file_paths {
                    let matches_pattern = match pattern_opt {
                        None => true,
                        Some(ref pattern) => {
                            match Regex::new(&pattern) {
                                Ok(reg_expr) => match reg_expr.captures(&file_path) {
                                    Some(_) => true,
                                    None => false,
                                },
                                Err(err) => return Err(Error::new(ErrorKind::Other, err)),
                            }
                        },
                    };
                    if matches_pattern {
                        match PictureEntry::from_file(&file_path) {
                            Ok(picture_entry) => self.picture_entries.push(picture_entry),
                            Err(err) => return Err(err),
                        }
                    }
                }
                Ok(())
            },
            Err(err) => Err(err),
        }
    }

    pub fn add_picture_entries_from_file_list(&mut self, file_list: &str) -> Result<()> {
        match read_to_string(file_list) {
            Err(err) => Err(err),
            Ok(content) => {
                for path in content.lines()
                    .map(String::from)
                        .filter(|p| !is_thumbnail(p))
                        .collect::<Vec<_>>()
                        .into_iter()
                        .map(|l| PathBuf::from(l)) {
                            let file_path = path.to_str().unwrap().to_string();
                            match PictureEntry::from_file(&file_path) {
                                Ok(picture_entry) => self.picture_entries.push(picture_entry),
                                Err(err) => return Err(err),
                            }
                        };
                Ok(())
            },
        }
    }

    pub fn add_picture_entry_from_file(&mut self, file_path: &str) -> Result<()> {
        match check_file(file_path) {
            Ok(_) => match PictureEntry::from_file(file_path) {
                Ok(picture_entry) => Ok(self.picture_entries.push(picture_entry)),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }

    // queries

    pub fn length(&self) -> usize {
        self.picture_entries.len()
    }

    pub fn last(&self) -> usize {
        self.length()-1
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

    pub fn palette_on(&self) -> bool {
        self.palette_on
    }

    pub fn full_size_on(&self) -> bool {
        self.full_size_on
    }


    pub fn index(&self) -> Option<usize> {
        if self.index < self.picture_entries.len() {
            Some(self.index)
        } else {
            None
        }
    }

    pub fn current_entry(&self) -> Option<&PictureEntry> {
        self.index().and_then(|index| Some(&self.picture_entries[index]))
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

    pub fn can_move_to_index(&self, index: usize) -> bool {
        index < self.picture_entries.len()
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

    pub fn set_page_size(&mut self, page_size: usize) {
        assert!(page_size > 0 && page_size <= 10);
        self.page_size = page_size;
    }

    pub fn toggle_page_limit(&mut self) {
        self.page_limit_on = !self.page_limit_on
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

    pub fn index_input_pattern(&mut self) -> Option<usize> {
        if let Some(pattern) = &self.input {
            self.picture_entries.iter().position(|entry| entry.original_file_name().contains(&*pattern))
        } else {
            None
        }
    }

    pub fn index_input_number(&mut self) -> Option<usize> {
        if let Some(number) = &self.input {
            let index = number.parse::<usize>().unwrap();
            if index < self.length() {
                Some(index)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn apply_label(&mut self, label: String) -> Result<()> {
        match self.index() {
            Some(index) => {
                let entry = &mut self.picture_entries[index];
                entry.set_label(label);
                entry.save_image_data()
            },
            None => Err(Error::new(ErrorKind::Other, "empty catalog"))
        }
    }
    

    pub fn set_label(&mut self) -> Result<()> {
        self.label = self.input.clone();
        self.input = None;
        match &self.label {
            Some(s) => self.apply_label(s.clone()),
            None => Ok(()),
        }
    }

    pub fn copy_label(&mut self) {
        if let Some(entry) = self.current_entry() {
            self.label = entry.label()
        }
    }

    pub fn paste_label(&mut self) -> Result<()> {
        match &self.label {
            Some(s) => self.apply_label(s.clone()),
            None => Ok(()),
        }
    }

    pub fn toggle_palette(&mut self) {
        self.palette_on = !self.palette_on
    }

    pub fn toggle_full_size(&mut self) {
        self.full_size_on = !self.full_size_on
    }

    pub fn start_set(&mut self) {
        if self.current_entry().is_some() {
            self.start_index = Some(self.index)
        }
    }

    pub fn cancel_set(&mut self) {
        if self.current_entry().is_some() {
            self.start_index = None
        }
    }
    pub fn unlabel(&mut self) -> Result<()> {
        match self.index() {
            Some(index) => {
                let entry: &mut PictureEntry = &mut self.picture_entries[index];
                entry.unlabel();
                entry.save_image_data()
            },
            None => Ok(()),
        }
    }

    pub fn end_set_label(&mut self) -> Result<()> {
        match self.index() {
            Some(index) => {
                if let Some(label) = &self.label {
                    match self.start_index {
                        None => self.set_label(),
                        Some(other) => {
                            let (start,end) = if other <= index { (other,index) } else { (index,other) };
                            for i in start..end+1 {
                                let entry: &mut PictureEntry = &mut self.picture_entries[i];
                                entry.set_label(label.to_string());
                                match entry.save_image_data() {
                                    Ok(()) => {},
                                    Err(err) => return Err(err),
                                }
                            };
                            Ok(())
                        },
                    }
                } else {
                    Ok(())
                }
            },
            None => Err(Error::new(ErrorKind::Other, "empty catalog")),
        }
    }

    pub fn end_unlabel(&mut self) -> Result<()> {
        match self.index() {
            Some(index) => {
                match self.start_index {
                    None => self.unlabel(),
                    Some(other) => {
                        let (start,end) = if other <= index { (other,index) } else { (index,other) };
                        for i in start..end+1 {
                            let entry: &mut PictureEntry = &mut self.picture_entries[i];
                            entry.unlabel();
                            match entry.save_image_data() {
                                Ok(()) => {},
                                Err(err) => return Err(err),
                            }
                        };
                        Ok(())
                    },
                }
            },
            None => Err(Error::new(ErrorKind::Other, "empty catalog")),
        }
    }
    pub fn select(&mut self) -> Result<()> {
        match self.index() {
            Some(index) => {
                let entry: &mut PictureEntry = &mut self.picture_entries[index];
                if !entry.deleted {
                    entry.selected = true;
                    entry.save_image_data()
                } else {
                    Ok(())
                }
            },
            None => Ok(())
        }
    }

    pub fn end_set_select(&mut self) -> Result<()> {
        match self.index() {
            Some(index) => {
                match self.start_index {
                    None => self.select(),
                    Some(other) => {
                        let (start,end) = if other <= index { (other,index) } else { (index,other) };
                        for i in start..end+1 {
                            let entry: &mut PictureEntry = &mut self.picture_entries[i];
                            entry.selected = true;
                            match entry.save_image_data() {
                                Ok(()) => {},
                                Err(err) => return Err(err),
                            }
                        };
                        Ok(())
                    },
                }
            },
            None => Err(Error::new(ErrorKind::Other, "empty catalog")),
        }
    }

    pub fn unselect_page(&mut self) -> Result<()> {
        match self.index() {
            Some(_) => {
                let start = self.page_index();
                let end = start + self.page_length();
                for i in start..end {
                    let entry: &mut PictureEntry = &mut self.picture_entries[i];
                    entry.selected = false;
                    match entry.save_image_data() {
                        Ok(()) => {},
                        Err(err) => return Err(err),
                    }
                };
                Ok(())
            },
            None => Err(Error::new(ErrorKind::Other, "empty catalog")),
        }
    }

    pub fn unselect_all(&mut self) -> Result<()> {
        match self.index() {
            Some(_) => {
                let start = 0;
                let end = self.length();
                for i in start..end {
                    let entry: &mut PictureEntry = &mut self.picture_entries[i];
                    entry.selected = false;
                    match entry.save_image_data() {
                        Ok(()) => {},
                        Err(err) => return Err(err),
                    }
                };
                Ok(())
            },
            None => Err(Error::new(ErrorKind::Other, "empty catalog")),
        }
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

    pub fn toggle_delete_entry(&mut self) -> Result<()> {
        if let Some(index) = self.index() {
            let entry = &mut self.picture_entries[index];
            entry.deleted = !entry.deleted;
            entry.save_image_data()
        } else {
            Ok(())
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
            make_picture_entry(String::from("testdata/foo.jpeg"), 100, 5, day_d, Rank::NoStar, None, Some(String::from("foo")), false),
            make_picture_entry(String::from("testdata/bar.jpeg"), 1000, 15, day_b, Rank::ThreeStars, None, None, false),
            make_picture_entry(String::from("testdata/qux.jpeg"), 10, 25, day_c, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None, false),
            make_picture_entry(String::from("testdata/bub.jpeg"), 100, 25, day_a, Rank::OneStar, None, Some(String::from("xanadoo")),false))
    }

    fn my_catalog() -> Catalog {
        let mut catalog = Catalog::new();
        let mut example = my_entries();
        catalog.add_picture_entries(&mut example);
        catalog
    }

    fn my_larger_catalog() -> Catalog {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let mut other_entries = vec![
            make_picture_entry(String::from("testdata/joe.jpeg"), 100, 5, day_a, Rank::NoStar, None, Some(String::from("foo")),false),
            make_picture_entry(String::from("testdata/gus.jpeg"), 1000, 15, day_a, Rank::ThreeStars, None, None, false),
            make_picture_entry(String::from("testdata/zoo.jpeg"), 10, 25, day_a, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None, false)];
        let mut catalog = my_catalog();
        catalog.add_picture_entries(&mut other_entries);
        catalog
    }

    #[test]
    fn at_creation_length_is_0() {
        let catalog = Catalog::new();
        assert_eq!(catalog.length(), 0);
        assert_eq!(true, catalog.current_entry().is_none());
    }

    #[test]
    fn after_adding_entries_length_is_updated() {
        let catalog = my_catalog();
        assert_eq!(catalog.length(), 4);
    }

    #[test]
    fn cannot_move_beyond_length() {
        let catalog = my_catalog();
        assert_eq!(false, catalog.can_move_to_index(4));
    }

    #[test]
    fn sorting_catalog_by_different_criteria() {
        let mut catalog = my_catalog();
        catalog.sort_by(Order::Size);
        catalog.move_to_index(0);
        assert_eq!(String::from("qux.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Date);
        catalog.move_to_index(0);
        assert_eq!(String::from("bub.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Name);
        catalog.move_to_index(0);
        assert_eq!(String::from("bar.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Colors);
        catalog.move_to_index(0);
        assert_eq!(String::from("foo.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Value);
        catalog.move_to_index(3);
        assert_eq!(String::from("foo.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Label);
        catalog.move_to_index(0);
        assert_eq!(String::from("foo.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.move_to_index(1);
        assert_eq!(String::from("bub.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Palette);
        catalog.move_to_index(3);
        assert_eq!(String::from("qux.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
    }

    #[test]
    fn page_index_depends_on_page_size_and_index() {
        let mut catalog = my_larger_catalog();
        catalog.set_page_size(2);
        assert_eq!(4, catalog.page_length());
        catalog.move_to_index(0);
        assert_eq!(0, catalog.page_index());
        catalog.move_to_index(6);
        assert_eq!(4, catalog.page_index());
    }

    #[test]
    fn after_moving_next_page_index_depends_on_page_size() {
        let mut catalog = my_larger_catalog();
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
        let mut catalog = my_larger_catalog();
        assert_eq!(7, catalog.length());
        assert_eq!(true, catalog.page_limit_on());
        catalog.set_page_size(2);
        catalog.move_to_index(0);
        catalog.move_towards(Direction::Right);
        assert_eq!(1, catalog.index().unwrap());
        catalog.move_towards(Direction::Down);
        assert_eq!(3, catalog.index().unwrap());
        catalog.move_towards(Direction::Left);
        assert_eq!(2, catalog.index().unwrap());
        assert_eq!(true, catalog.can_move_towards(Direction::Up));
        catalog.move_towards(Direction::Up);
        assert_eq!(0, catalog.index().unwrap());
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
        assert_eq!(3, catalog.index().unwrap());
        catalog.move_towards(Direction::Down);
        assert_eq!(5, catalog.index().unwrap());
        assert_eq!(true, catalog.can_move_towards(Direction::Down));
        catalog.move_towards(Direction::Down);
        assert_eq!(0, catalog.index().unwrap());
        catalog.move_towards(Direction::Right);
        assert_eq!(1, catalog.index().unwrap());
        assert_eq!(true, catalog.can_move_towards(Direction::Up));
        catalog.move_towards(Direction::Up);
        assert_eq!(6, catalog.index().unwrap()); // because there's no picture entry in pos 7
        catalog.move_to_index(5);
        assert_eq!(true, catalog.can_move_towards(Direction::Down));
        catalog.move_towards(Direction::Down);
        assert_eq!(0, catalog.index().unwrap()); // because there's no picture entry in pos 7
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
        assert_eq!(String::from("qux.jpeg"),catalog.current_entry().unwrap().original_file_name());
        catalog.begin_input();
        catalog.add_input_char('f');
        catalog.add_input_char('o');
        let index = catalog.index_input_pattern();
        assert_eq!(true, index.is_some());
        catalog.move_to_index(index.unwrap());
        assert_eq!(String::from("foo.jpeg"), catalog.current_entry().unwrap().original_file_name());
        catalog.begin_input();
        catalog.add_input_char('q');
        catalog.add_input_char('a');
        assert_eq!(None, catalog.index_input_pattern());
    }

    #[test]
    fn index_of_entry_by_input() {
        let mut example = my_entries();
        let mut catalog = Catalog::new();
        catalog.add_picture_entries(&mut example);
        catalog.sort_by(Order::Size);
        catalog.move_to_index(0);
        catalog.begin_input();
        catalog.add_input_char('3');
        let index = catalog.index_input_number();
        assert_eq!(true, index.is_some());
        catalog.move_to_index(index.unwrap());
        assert_eq!(String::from("bar.jpeg"), catalog.current_entry().unwrap().original_file_name());
        catalog.add_input_char('3');
        let wrong = catalog.index_input_number();
        assert_eq!(true, wrong.is_none());
    }

    #[test]
    fn state_indicators() {
        let mut catalog = Catalog::new();
        assert_eq!(false, catalog.palette_on());
        assert_eq!(false, catalog.full_size_on());
        catalog.toggle_palette();
        catalog.toggle_full_size();
        assert_eq!(true, catalog.palette_on());
        assert_eq!(true, catalog.full_size_on());

    }

    #[test]
    fn label_entry() {
        let mut catalog = my_catalog();
        catalog.move_to_index(1);
        assert_eq!(None, catalog.current_entry().unwrap().label());
        catalog.begin_input();
        assert_eq!(true, catalog.input_on());
        catalog.add_input_char('R');
        catalog.add_input_char('E');
        catalog.add_input_char('X');
        let _ = catalog.set_label();
        assert_eq!(Some(String::from("REX")), catalog.current_entry().unwrap().label());
        assert_eq!(false, catalog.input_on());
        catalog.move_to_index(0);
        assert_eq!(Some(String::from("foo")), catalog.current_entry().unwrap().label());
        catalog.copy_label();
        catalog.move_to_index(1);
        assert_eq!(Some(String::from("REX")), catalog.current_entry().unwrap().label());
        let _ = catalog.paste_label();
        assert_eq!(Some(String::from("foo")), catalog.current_entry().unwrap().label());
    }

    #[test]
    fn label_and_unlabel_entries() {
        let mut catalog = my_catalog();
        catalog.move_to_index(0);
        catalog.copy_label();
        catalog.start_set();
        catalog.move_to_index(3);
        assert_eq!(true, catalog.end_set_label().is_ok());
        catalog.move_to_index(1);
        assert_eq!(Some(String::from("foo")), catalog.current_entry().unwrap().label());
        catalog.move_to_index(1);
        assert_eq!(Some(String::from("foo")), catalog.current_entry().unwrap().label());
        catalog.move_to_index(2);
        assert_eq!(Some(String::from("foo")), catalog.current_entry().unwrap().label());
        catalog.move_to_index(0);
        catalog.cancel_set();
        catalog.start_set();
        catalog.move_to_index(2);
        assert_eq!(true, catalog.end_unlabel().is_ok());
        catalog.move_to_index(1);
        assert_eq!(None, catalog.current_entry().unwrap().label());
        catalog.move_to_index(1);
        assert_eq!(None, catalog.current_entry().unwrap().label());
        catalog.move_to_index(2);
        assert_eq!(None, catalog.current_entry().unwrap().label());
        catalog.move_to_index(3);
        assert_eq!(true, catalog.unlabel().is_ok());
        assert_eq!(None, catalog.current_entry().unwrap().label());
    }

    #[test]
    fn select_and_unselect_entries() {
        let mut catalog = my_larger_catalog();
        catalog.set_page_size(2);
        catalog.move_to_index(0);
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        catalog.copy_label();
        catalog.start_set();
        catalog.move_to_index(6);
        assert_eq!(true, catalog.end_set_select().is_ok());
        catalog.move_to_index(0);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(1);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(2);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(3);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(4);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(5);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(6);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(2);
        assert_eq!(true, catalog.unselect_page().is_ok());
        catalog.move_to_index(4);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(5);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(6);
        assert_eq!(true, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(0);
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(1);
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(2);
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(3);
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        assert_eq!(true, catalog.unselect_all().is_ok());
        catalog.move_to_index(4);
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(5);
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        catalog.move_to_index(6);
        assert_eq!(false, catalog.current_entry().unwrap().selected);
    }

    #[test]
    fn deleting_en_entry_makes_it_non_selectable() {
        let mut catalog = my_catalog();
        catalog.move_to_index(0);
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        assert_eq!(true, catalog.toggle_delete_entry().is_ok());
        assert_eq!(true, catalog.current_entry().unwrap().deleted);
        let _ = catalog.select();
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        assert_eq!(true, catalog.toggle_delete_entry().is_ok());
        assert_eq!(false, catalog.current_entry().unwrap().deleted);
        let _ = catalog.select();
        assert_eq!(true, catalog.current_entry().unwrap().selected);
    }

    #[test] 
    fn adding_entries_from_a_directory() {
        let mut catalog = Catalog::new();
        let result = catalog.add_picture_entries_from_dir("testdata", None);
        assert_eq!(true, result.is_ok());
        assert_eq!(10, catalog.length())
    }

    #[test] 
    fn adding_entries_from_a_directory_with_pattern_option() {
        let mut catalog = Catalog::new();
        let result = catalog.add_picture_entries_from_dir("testdata", Some(String::from("or")));
        assert_eq!(true, result.is_ok());
        assert_eq!(2, catalog.length());
        assert_eq!(String::from("labrador.jpg"), catalog.picture_entries[0].original_file_name());
        assert_eq!(String::from("color-wheel.png"), catalog.picture_entries[1].original_file_name());
    }

    #[test]
    fn adding_entry_from_a_single_file() {
        let mut catalog = Catalog::new();
        let result = catalog.add_picture_entry_from_file("testdata/color-wheel.png");
        assert_eq!(true, result.is_ok());
        assert_eq!(1, catalog.length());
        assert_eq!(String::from("color-wheel.png"), catalog.picture_entries[0].original_file_name());
    }

    #[test]
    fn adding_entries_from_a_file_list() {
        let mut catalog = Catalog::new();
        let result = catalog.add_picture_entries_from_file_list("testdata/selection");
        assert_eq!(true, result.is_ok());
        assert_eq!(3, catalog.length());
        assert_eq!(String::from("3-cubes.jpeg"), catalog.picture_entries[0].original_file_name());
        assert_eq!(String::from("ChessSet.jpg"), catalog.picture_entries[1].original_file_name());
        assert_eq!(String::from("cumulus.jpeg"), catalog.picture_entries[2].original_file_name());
    }
}
