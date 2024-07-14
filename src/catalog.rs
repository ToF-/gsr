use crate::picture_entry::PictureEntry;

pub struct Catalog {
    picture_entries: Vec<PictureEntry>,
}

impl Catalog {

    pub fn new() -> Self {
        Catalog {
            picture_entries : Vec::new(),
        }
    }

    pub fn length(&self) -> usize {
        self.picture_entries.len()
    }

    pub fn add_picture_entries(&mut self, picture_entries: &mut Vec<PictureEntry>) {
        self.picture_entries.append(picture_entries)
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
            make_picture_entry(String::from("photos/foo.jpeg"), 100, 5, day_d, Rank::NoStar),
            make_picture_entry(String::from("photos/bar.jpeg"), 1000, 15, day_b, Rank::ThreeStars),
            make_picture_entry(String::from("photos/qux.jpeg"), 10, 25, day_c, Rank::TwoStars),
            make_picture_entry(String::from("photos/bub.jpeg"), 100, 25, day_a, Rank::OneStar))
    }

    #[test]
    fn at_creation_length_is_0() {
        let catalog = Catalog::new();
        assert_eq!(catalog.length(), 0);
    }

    #[test]
    fn after_adding_entries_length_is_updated() {
        let mut catalog = Catalog::new();
        let mut example = my_entries();
        catalog.add_picture_entries(&mut example);
        assert_eq!(catalog.length(), 4);
    }
}
