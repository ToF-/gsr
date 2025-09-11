use rand::Rng;
use rand::thread_rng;
use crate::direction::Direction;
use crate::catalog::Coords;

#[derive(Debug,Clone)]
pub struct Navigator {
    done: bool,
    index: usize,
    previous_index: Option<usize>,
    length: usize,
    new_page_size: Option<usize>,
    page_changed: bool,
    page_limit_on: bool,
    page_size: usize,
    start_index: Option<usize>,
}

impl Navigator {

    pub fn new() -> Self {
        Navigator {
            done: false,
            index: 0,
            previous_index: None,
            length: 0,
            new_page_size: None,
            page_changed: false,
            page_limit_on: false,
            page_size: 1,
            start_index: None,
        }
    }

    pub fn page_limit_on(&self) -> bool {
        self.page_limit_on
    }

    pub fn toggle_page_limit(&mut self) {
        self.page_limit_on = !self.page_limit_on
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
    pub fn set_length(&mut self, length: usize) {
        self.length = length
    }

    pub fn set_new_page_size(&mut self, page_size: usize) {
        assert!(page_size > 0 && page_size <= 10);
        self.new_page_size = Some(page_size);
        self.previous_index = Some(self.index)
    }

    pub fn move_to_index(&mut self, index: usize) {
        if index != self.index {
            self.index = index;
        }
    }

    pub fn move_to_first_index(&mut self) {
        self.move_to_index(0)
    }

    pub fn move_to_last_index(&mut self) {
        self.move_to_index(self.length - 1)
    }

    pub fn move_to_previous_index(&mut self) {
        match self.previous_index {
            Some(index) => self.move_to_index(index),
            None => self.move_to_index(0),
        }
    }

    pub fn move_next_page(&mut self) {
        let candidate_index = self.page_index() + self.page_length();
        self.move_to_index( if candidate_index < self.length() { candidate_index } else { 0 });
    }

    pub fn move_prev_page(&mut self) {
        let index = if self.page_index() >= self.page_length() {
            self.page_index() - self.page_length()
        } else {
            self.page_index_of(self.length()-1)
        };
        self.move_to_index(index);
    }
    pub fn exit(&mut self) {
        self.done = true
    }

    pub fn last(&self) -> usize {
        self.length - 1
    }
    pub fn move_towards(&mut self, direction: Direction) {
        if self.can_move_towards(direction.clone()) {
            let mut index = self.index().expect("incorrect index value");
            match direction {
                Direction::Right => if index + 1 < self.length() { index += 1 },
                Direction::Left => if index > 0 { index -= 1 },
                Direction::Down => if index + self.page_size() < self.length() { index += self.page_size() } else { index = 0 },
                Direction::Up => {
                    if index >= self.page_size() {
                        index -= self.page_size()
                    } else {
                        let offset = index - self.page_index();
                        let new_page_index = self.last() - (self.last() % self.page_length());
                        let new_index = new_page_index + self.page_length() - self.page_size() + offset;
                        index = if new_index < self.length() {
                            new_index
                        } else {
                            self.last()
                        }
                    }
                },
            };
            self.move_to_index(index);
        }
    }
    pub fn done(&self) -> bool {
        self.done
    }

    pub fn start_set(&mut self) {
        match self.index() {
            Some(index) => { self.start_index = Some(index) },
            None => {},
        }
    }

    pub fn cancel_set(&self) -> Self {
        Navigator {
            start_index: None,
            ..self.clone()
        }
    }

    pub fn set_page_size(&mut self, page_size: usize) {
        assert!(page_size > 0 && page_size <= 10);
        self.page_size =  page_size
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
            index,
            ..self.clone()
        }
    }

    pub fn can_move_towards(&self, direction: Direction) -> bool {
        let index = self.index().expect("incorrect index value");
        let cells_per_row = self.page_size();
        let col = index % cells_per_row;
        let row = (index - self.page_index()) / cells_per_row;
        if self.page_limit_on {
            match direction {
                Direction::Left => col > 0,
                Direction::Right => col+1 < cells_per_row && index+1 < self.length(),
                Direction::Up => row > 0,
                Direction::Down => row+1 < cells_per_row && index + cells_per_row < self.length(),
            }
        } else {
            true
        }
    }

    pub fn move_to_random_index(&mut self) {
        let index = thread_rng().gen_range(0..self.length());
        if self.can_move_to_index(index) {
            self.move_to_index(index)
        }
    }

    pub fn can_move_to_index(&self, index: usize) -> bool {
        index < self.length
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

#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn cannot_move_beyond_length() {
        let navigator = Navigator::new();
        assert_eq!(false, navigator.can_move_to_index(4));
    }

    #[test]
    fn page_index_depends_on_page_size_and_index() {
        let mut navigator = Navigator::new();
        navigator.set_length(7);
        navigator.set_page_size(2);
        assert_eq!(4, navigator.page_length());
        navigator.move_to_index(0);
        assert_eq!(0, navigator.page_index());
        navigator.move_to_index(6);
        assert_eq!(4, navigator.page_index());
    }


    #[test]
    fn after_moving_next_page_index_depends_on_page_size() {
        let mut navigator = Navigator::new();
        navigator.set_length(7);
        navigator.set_page_size(2);
        navigator.move_to_index(2);
        navigator.move_next_page();
        assert_eq!(4, navigator.page_index());
        navigator.move_next_page();
        assert_eq!(0, navigator.page_index());
        navigator.move_prev_page();
        assert_eq!(4, navigator.page_index());
    }

    #[test]
    fn moving_next_picture_can_be_blocked_or_allowed() {
        let mut navigator = Navigator::new();
        navigator.set_length(7);
        navigator.toggle_page_limit();
        assert_eq!(true, navigator.page_limit_on);
        navigator.set_page_size(2);
        navigator.move_to_index(0);
        navigator.move_towards(Direction::Right);
        assert_eq!(1, navigator.index().unwrap());
        assert_eq!(4, navigator.page_length());
        assert_eq!(true, navigator.can_move_towards(Direction::Down));
        navigator.move_towards(Direction::Down);
        assert_eq!(3, navigator.index().unwrap());
        navigator.move_towards(Direction::Left);
        assert_eq!(2, navigator.index().unwrap());
        assert_eq!(true, navigator.can_move_towards(Direction::Up));
        navigator.move_towards(Direction::Up);
        assert_eq!(0, navigator.index().unwrap());
        assert_eq!(false, navigator.can_move_towards(Direction::Left));
        assert_eq!(false, navigator.can_move_towards(Direction::Up));
        assert_eq!(true, navigator.can_move_towards(Direction::Right));
        assert_eq!(true, navigator.can_move_towards(Direction::Down));
        navigator.move_towards(Direction::Right);
        assert_eq!(false, navigator.can_move_towards(Direction::Right));
        navigator.move_towards(Direction::Down);
        assert_eq!(false, navigator.can_move_towards(Direction::Down));
        navigator.toggle_page_limit();
        assert_eq!(false, navigator.page_limit_on);
        assert_eq!(3, navigator.index().unwrap());
        navigator.move_towards(Direction::Down);
        assert_eq!(5, navigator.index().unwrap());
        assert_eq!(true, navigator.can_move_towards(Direction::Down));
        navigator.move_towards(Direction::Down);
        assert_eq!(0, navigator.index().unwrap());
        navigator.move_towards(Direction::Right);
        assert_eq!(1, navigator.index().unwrap());
        assert_eq!(true, navigator.can_move_towards(Direction::Up));
        navigator.move_towards(Direction::Up);
        assert_eq!(6, navigator.index().unwrap()); // because there's no picture entry in pos 7
        navigator.move_to_index(5);
        assert_eq!(true, navigator.can_move_towards(Direction::Down));
        navigator.move_towards(Direction::Down);
        assert_eq!(0, navigator.index().unwrap()); // because there's no picture entry in pos 7
    }
}
