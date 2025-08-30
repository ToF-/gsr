use crate::loader::load_picture_entries_from_source;
use crate::picture_entry::PictureEntries;
use crate::loader::load_picture_entries_from_db;
use crate::display::title_display;
use anyhow::{anyhow, Result};
use crate::comment::Comment;
use crate::args::Args;
use crate::database::Database;
use crate::direction::Direction;
use crate::editor::{Editor};
use crate::order::Order;
use crate::path::file_name;
use crate::path::standard_directory;
use crate::path::{get_picture_file_paths, file_path_directory};
use crate::picture_entry::PictureEntry;
use crate::picture_io::{append_to_extract_file, check_or_create_thumbnail_file};
use crate::rank::Rank;
use rand::Rng;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use std::cmp::Ordering::{Less, Greater, Equal};
use std::collections::HashMap;
use std::collections::HashSet;
use std::process::exit;

pub type Coords = (usize, usize);

#[derive(Debug)]
pub struct Catalog {
    db_centric: bool,
    picture_entries: Vec<PictureEntry>,
    index: usize,
    page_size: usize,
    page_limit_on: bool,
    copied_label: Option<String>,
    last_comment: Option<Comment>,
    palette_on: bool,
    full_size_on: bool,
    expand_on: bool,
    start_index: Option<usize>,
    page_changed: bool,
    order: Option<Order>,
    max_selected: usize,
    previous_order: Option<Order>,
    args: Option<Args>,
    discarded: Vec<usize>,
    database: Database,
    pub tags: HashSet<String>,
    current_candidates: Vec<String>,
}

impl Catalog {

    // creation

    pub fn new() -> Self {
        Catalog {
            db_centric: false,
            picture_entries : Vec::new(),
            index: 0,
            page_size: 1,
            page_limit_on: false,
            copied_label: None,
            last_comment: None,
            palette_on: false,
            full_size_on: false,
            expand_on: false,
            start_index: None,
            page_changed: false,
            order: Some(Order::Random),
            max_selected: 0,
            previous_order: Some(Order::Random),
            args: None,
            discarded: Vec::new(),
            database: match Database::initialize(true) {
                Ok(database) => database,
                Err(err) => {
                    eprintln!("{}", err);
                    exit(1);
                }
            },
            tags: HashSet::new(),
            current_candidates: vec![],
        }
    }


    pub fn init_catalog(args: &Args) -> Result<Self> {
        let mut catalog = Self::new();
        catalog.args = Some(args.clone());
        catalog.set_page_size(catalog.args.clone().unwrap().grid.unwrap());
        let mut database = Database::initialize(false).unwrap();
        let add_result = Catalog::set_picture_entries(&mut catalog, load_picture_entries_from_source(&mut database, args));
        if let Err(err) = add_result {
            return Err(err)
        };
        catalog.count_selected();
        if catalog.length() == 0 {
            return Err(anyhow!("no picture to show"))
        };
        match catalog.initialize_tags() {
            Ok(()) => Ok(catalog),
            Err(err) => Err(anyhow!(err))
        }
    }

    #[allow(dead_code)]
    pub fn add_picture_entries(&mut self, entries: &Vec<PictureEntry>) {
        for entry in entries {
            self.picture_entries.push(entry.clone())
        }
    }

    pub fn initialize_tags(&mut self) -> Result<()> {
        self.tags = HashSet::new();
        match self.database.load_all_tags() {
            Ok(labels) => {
                for label in labels {
                    self.tags.insert(label);
                };
                Ok(())
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn deduplicate_files(&mut self, target_dir: &str) -> Result<()> {
        self.sort_by(Order::Size);
        let mut prev_entry: Option<PictureEntry> = None;
        for entry in &self.picture_entries {
            if let Some(prev) = prev_entry {
                match entry.equal_content(&prev) {
                    Ok(true) => {
                        println!("removing duplicate entry {}, same as {}", prev.original_file_path(), entry.original_file_path());
                        match prev.copy_files(target_dir) {
                            Ok(_) => match prev.delete_files() {
                                Ok(()) => {},
                                Err(err) => return Err(anyhow!(err)),
                            },
                            Err(err) => return Err(anyhow!(err)),
                        }
                    },
                    Ok(false) => {} ,
                    Err(err) => return Err(anyhow!(err)),
                }
            };
            prev_entry = Some(entry.clone());
        };
        Ok(())
    }
    pub fn update_files(&self) -> Result<()> {
        let mut update_result = Ok(());
        for entry in &self.picture_entries {
            let result = check_or_create_thumbnail_file(&entry.thumbnail_file_path(), &entry.original_file_path());
            if result.is_err() {
                update_result = result;
                break
            }
        };
        update_result
    }

    pub fn file_operations(&self, also_fs:bool) -> Result<()> {
        self.delete_files(also_fs)
    }

    fn delete_files(&self, also_fs:bool) -> Result<()> {
        for entry in self.picture_entries.iter().filter(|e| e.deleted) {
            match self.database.delete_picture(entry.file_path.clone()) {
                Ok(()) => {},
                Err(err) => eprintln!("{}", err),
            };
            if also_fs {
                match entry.delete_files() {
                    Ok(()) => {},
                    Err(err) => eprintln!("{}", err),
                }
            };
        }
        Ok(())
    }

    fn delete_broken_entries_from_db(&mut self) -> Result<()> {
        match self.database.delete_difference_from_file_system() {
            Ok(file_paths) => {
                if self.picture_entries.len() > 0 {
                    let mut picture_entries: Vec<PictureEntry> = vec![];
                    let mut i = 0;
                    while i < self.picture_entries.len() {
                        if !file_paths.contains(&self.picture_entries[i].file_path) {
                            picture_entries.push(self.picture_entries[i].clone());
                        }
                        i+= 1;
                    };
                    self.picture_entries = picture_entries;
                };
                Ok(())
            },
            Err(err) => Err(anyhow!(err)),
        }
    }






    // queries

    pub fn copied_label(&self) -> Option<String> {
        self.copied_label.clone()
    }
    pub fn picture_entries(&self) -> &Vec<PictureEntry> {
        &self.picture_entries
    }
    
    pub fn args(&self) -> Option<Args> {
        self.args.clone()
    }

    pub fn db_centric(&self) -> bool {
        self.db_centric
    }

    pub fn max_selected(&self) -> usize {
        self.max_selected
    }

    pub fn start_index(&self) -> Option<usize> {
        self.start_index
    }

    pub fn order(&self) -> Option<Order> {
        self.order
    }

    pub fn discarded(&self) -> &Vec<usize> {
        &self.discarded
    }

    pub fn sort_selection_on(&self) -> bool {
        self.order.is_none()
    }

    pub fn cells_per_row(&self) -> usize {
        self.page_size
    }

    pub fn expand_on(&self) -> bool {
        self.expand_on
    }

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

    pub fn position_from_index(&self, index: usize) -> Coords {
        let start_index = self.page_index_of(index);
        let offset = index - start_index;
        let row = offset / self.cells_per_row();
        let col = offset % self.cells_per_row();
        (col, row)
    }

    pub fn index_from_position(&self, coords: Coords) -> Option<usize> {
        let index = (self.page_index() + coords.0 as usize + coords.1 as usize * self.page_size) as usize;
        if index < self.length() {
            Some(index)
        } else {
            None
        }
    }

    pub fn entry_at_index(&self, index: usize) -> Option<&PictureEntry> {
        if index < self.picture_entries.len() {
            Some(&self.picture_entries[index])
        } else {
            None
        }
    }

    pub fn current_entry(&self) -> Option<&PictureEntry> {
        self.index().and_then(|index| { self.entry_at_index(index) } )
    }

    pub fn page_index_of(&self, index: usize) -> usize {
        index - (index % self.page_length())
    }

    pub fn page_index(&self) -> usize {
        self.page_index_of(self.index)
    }

    pub fn current_candidates(&self) -> String {
        format!("{}", self.current_candidates.join(","))
    }
    pub fn can_move_to_index(&self, index: usize) -> bool {
        index < self.picture_entries.len()
    }

    pub fn can_move_towards(&self, direction: Direction) -> bool {
        let index = self.index;
        let cells_per_row = self.page_size;
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

    pub fn index_input_pattern(&mut self, pattern: &str) -> Option<usize> {
        self.picture_entries.iter().position(|entry|
            entry.original_file_path().contains(&*pattern))
    }

    pub fn index_label_search(&mut self, pattern: &str) -> Option<usize> {
        self.picture_entries.iter().position(|entry|
            if let Some(label) = entry.label() {
                label.contains(pattern)
            } else {
                false
            })
    }

    pub fn page_changed(&self) -> bool {
        self.page_changed
    }


    pub fn copy_picture_file_to_temp(&self) -> Result<()> {
        let entry = self.current_entry().unwrap();
        match entry.copy_picture_file_to_temp() {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn extract(&self) {
        let entry = self.current_entry().unwrap();
        let file_path = entry.original_file_path();
        if let Some(args) = &self.args {
            if let Some(extract_file_path) = &args.extract {
                match append_to_extract_file(&file_path, &extract_file_path) {
                    Ok(_) => {
                        println!("appended {} in {}", file_path, extract_file_path)
                    },
                    Err(err) => {
                        eprintln!("{}", err)
                    },
                }
            } else {
                eprintln!("no extract file path info");
            }
        } else {
            eprintln!("no args value");
        }

    }

    pub fn print_info(&self, editor: &Editor) {
        println!("{}", title_display(self, editor));
        println!("{}", self.current_entry().expect("can't access current entry").original_file_path());
        println!("{:?}", self.current_entry().expect("can't access current entry"));
    }
    // updates
    
    pub fn set_current_picture_entry(&mut self, picture_entry: PictureEntry) -> Result<()> {
        match self.index() {
            Some(index) => {
                self.picture_entries[index] = picture_entry.clone();
                self.database.update_image_data(&picture_entry)
            },
            None => Ok(())
        }
    }

    pub fn set_picture_entries(&mut self, picture_entries_result: Result<PictureEntries>) -> Result<()> {
        match picture_entries_result {
            Ok(picture_entries) => {
                self.picture_entries = picture_entries;
                Ok(())
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn cover(&mut self) -> Result<()> {
        let entry = self.picture_entries[self.index].clone();
        let dir_path = file_path_directory(&entry.file_path);
        let file_name = file_name(&entry.file_path);
        let rank = entry.rank;
        println!("cover for {} with image {} and rank {}", dir_path, file_name, rank );
        match self.database.delete_cover(&dir_path, &file_name) {
            Ok(()) => match self.database.insert_cover(&dir_path, &file_name, rank) {
                Ok(()) => Ok(()),
                Err(err) => Err(anyhow!(err)),
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn label_current_entry(&mut self, label: &str) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.set_label(label);
                match self.set_current_picture_entry(new_picture_entry) {
                    Ok(()) => {
                        self.copied_label = Some(label.to_string());
                        self.tags.insert(label.to_string());
                        self.last_comment = Some(Comment::Label { label: label.to_string() });
                        Ok(())
                    },
                    Err(err) => Err(anyhow!(err)),
                } 
            },
            None => Ok(()),
        }
    }

    pub fn unlabel_current_entry(&mut self) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.unlabel();
                match self.set_current_picture_entry(new_picture_entry) {
                    Ok(()) => {
                        self.last_comment = Some(Comment::Unlabel);
                        Ok(())
                    },
                    Err(err) => Err(anyhow!(err)),
                } 
            },
            None => Ok(()),
        }
    }

    pub fn paste_label_current_entry(&mut self) -> Result<()> {
        match self.copied_label() {
            Some(label) => match self.current_entry() {
                Some(picture_entry) => {
                    let mut new_picture_entry = picture_entry.clone();
                    new_picture_entry.set_label(&label);
                    match self.set_current_picture_entry(new_picture_entry) {
                        Ok(()) => {
                            self.last_comment = Some(Comment::Label { label: label } );
                            Ok(())
                        },
                        Err(err) => Err(anyhow!(err)),
                    } 
                },
                None => Ok(()),
            },
            None => Ok(()),
        }
    }

    pub fn tag_current_entry(&mut self, tag: &str) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.add_tag(tag);
                match self.set_current_picture_entry(new_picture_entry) {
                    Ok(()) => {
                        self.tags.insert(tag.to_string());
                        self.last_comment = Some(Comment::AddTag { label: tag.to_string() });
                        Ok(())
                    },
                    Err(err) => Err(anyhow!(err)),
                } 
            },
            None => Ok(()),
        }
    }

    pub fn delete_tag(&mut self, tag: &str) -> Result<()> {
        let entry = &mut self.picture_entries[self.index];
        self.last_comment = Some(Comment::DeleteTag { label: tag.to_string() });
        if entry.tags.contains(tag) {
            entry.delete_tag(tag);
            self.database.delete_tag_label(entry, tag)
        } else {
            println!("tag not found: {}", tag);
            Ok(())
        }
    }

    pub fn toggle_select_current_entry(&mut self) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.selected = !new_picture_entry.selected;
                match self.set_current_picture_entry(new_picture_entry) {
                    Ok(()) => {
                        self.last_comment = Some(Comment::Select);
                        Ok(())
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            None => Ok(()),
        }
    }

    pub fn begin_sort_selection(&mut self) {
        self.previous_order = self.order.clone();
        self.order = None
    }

    pub fn cancel_sort_selection(&mut self) {
        self.order = self.previous_order.clone()
    }

    pub fn toggle_expand(&mut self) {
        self.expand_on = !self.expand_on;
        self.page_changed = true
    }

    pub fn delete(&mut self) {
        if let Some(index) = self.index() {
            self.picture_entries[index].deleted = !self.picture_entries[index].deleted;
            self.page_changed = true;
            self.last_comment = Some(Comment::Delete)
        }
    }
    pub fn refresh(&mut self) {
        self.page_changed = true
    }

    pub fn set_page_size(&mut self, page_size: usize) {
        assert!(page_size > 0 && page_size <= 10);
        self.page_size = page_size;
    }

    pub fn toggle_page_limit(&mut self) {
        self.page_limit_on = !self.page_limit_on
    }

    pub fn move_to_input_pattern(&mut self, pattern: &str) {
        match self.index_input_pattern(pattern) {
            Some(index) => if self.can_move_to_index(index) {
                self.move_to_index(index)
            },
            None => {},
        }
    }

    pub fn move_to_label_pattern(&mut self, pattern: &str) {
        println!("move_to_label_pattern({})", pattern);
        match self.index_label_search(pattern) {
            Some(index) => if self.can_move_to_index(index) {
                self.move_to_index(index)
            },
            None => {},
        }
    }

    pub fn start_set(&mut self) {
        if self.current_entry().is_some() {
            self.start_index = Some(self.index);
            println!("start set:{}", self.start_index.unwrap());
        }
    }

    pub fn cancel_set(&mut self) {
        if self.current_entry().is_some() {
            self.start_index = None
        }
    }

    pub fn rank_current_entry(&mut self, rank: Rank) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.set_rank(rank);
                match self.set_current_picture_entry(new_picture_entry) {
                    Ok(()) => {
                        self.last_comment = Some(Comment::Rank { rank: rank });
                        Ok(())
                    },
                    Err(err) => Err(anyhow!(err)),
                } 
            },
            None => Ok(()),
        }
    }

    pub fn label_tag_current_entry(&mut self) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => match picture_entry.label() {
                Some(label) => {
                    self.tag_current_entry(&label)
                },
                None => Ok(()),
            },
            None => Ok(()),
        }
    }

    pub fn apply_label_all(&mut self, label:&str) -> Result<()> {
        for i in 0..self.picture_entries.len() {
            self.index = i;
            self.last_comment = Some(Comment::Label { label: label.to_string() });
            match self.label_current_entry(label) {
                Ok(()) => {},
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
        
    pub fn print_labels_all(&mut self) -> Result<()> {
        let mut tags:HashMap<String,usize> = HashMap::new();

        for i in 0..self.picture_entries.len() {
            self.index = i;
            let entry = &self.picture_entries[i];
            if entry.label().is_some() {
                let stat = tags.entry(entry.label().unwrap().clone()).or_insert(0);
                *stat += 1;
            }
            match self.database.entry_tags(&entry.file_path) {
                Ok(labels) => {
                    for label in labels {
                        let stat = tags.entry(label.to_string()).or_insert(0);
                        *stat += 1;
                    }
                },
                Err(err) => return Err(anyhow!(err)),
            }
        };
        let mut stats:Vec<(usize,String)> = vec![];
        for (tag,stat) in tags {
            stats.push((stat,tag.clone()));
        };
        stats.sort_by(|a, b| {
            match (a.0).cmp(&b.0) {
                Greater => Less,
                Less => Greater,
                Equal => (a.1).cmp(&b.1),
                }});
        for (stat,tag) in stats {
            println!("{tag}:{stat}");
        };
        Ok(())
    }

    pub fn print_directories_all(self) -> Result<()> {
        match self.database.load_directories() {
            Ok(directories) => {
                for directory in directories {
                    println!("{}:{:6}", directory.0, directory.1)
                };
                Ok(())
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn repeat_last_comment(&mut self) -> Result<()> {
        match self.last_comment.clone() {
            None => Ok(()),
            Some(Comment::Label { label }) => self.label_current_entry(&label),
            Some(Comment::Unlabel) => self.unlabel(),
            Some(Comment::AddTag { label}) => self.tag_current_entry(&label),
            Some(Comment::DeleteTag { label}) => self.delete_tag(&label),
            Some(Comment::Rank { rank }) => self.rank_current_entry(rank),
            Some(Comment::Select) => self.toggle_select_current_entry(),
            Some(Comment::Delete) => Ok(self.delete()),
        }
    }

    pub fn set_selected_labels_with_input(&mut self, label: &str) -> Result<()> {
        for index in 0..self.picture_entries.len() {
            let entry = &mut self.picture_entries[index];
            if entry.selected {
                entry.set_label(label);
                match self.database.update_image_data(entry) {
                    Ok(()) => {},
                    Err(err) => return Err(anyhow!(err)),
                }
            }
        };
        Ok(())
    }

    pub fn copy_label(&mut self) {
        if let Some(entry) = self.current_entry() {
            self.copied_label = entry.label()
        }
    }

    pub fn toggle_palette(&mut self) {
        self.palette_on = !self.palette_on
    }

    pub fn toggle_full_size(&mut self) {
        self.full_size_on = !self.full_size_on
    }

    pub fn unlabel(&mut self) -> Result<()> {
        match self.index() {
            Some(index) => {
                let entry: &mut PictureEntry = &mut self.picture_entries[index];
                entry.unlabel();
                self.last_comment = Some(Comment::Unlabel);
                self.database.update_image_data(entry)
            },
            None => Ok(()),
        }
    }

    pub fn end_repeat_last_comment(&mut self) -> Result<()> {
        match self.index() {
            Some(index) => {
                println!("end set: {}, last comment: {:?}", index, &self.last_comment);
                match self.start_index {
                    None => self.repeat_last_comment(),
                    Some(other) => {
                        let (start,end) = if other <= index { (other,index) } else { (index,other) };
                        for i in start..end+1 {
                            let entry: &mut PictureEntry = &mut self.picture_entries[i];
                            match &self.last_comment {
                                None => {},
                                Some(Comment::Label { label }) => entry.set_label(&label),
                                Some(Comment::Unlabel) => entry.unlabel(),
                                Some(Comment::AddTag { label}) => entry.add_tag(&label),
                                Some(Comment::DeleteTag { label}) => entry.delete_tag(&label),
                                Some(Comment::Rank { rank }) => entry.set_rank(*rank),
                                Some(Comment::Select) => { entry.selected = !entry.selected }
                                Some(Comment::Delete) => { entry.deleted = !entry.deleted },
                            };
                            if self.last_comment.is_some() {
                                match self.database.update_image_data(&entry.clone()) {
                                    Ok(()) => {},
                                    Err(err) => return Err(anyhow!(err)),
                                }
                            };
                        };
                        self.start_index = None;
                        Ok(())
                    },
                }
            },
            None => Err(anyhow!("empty catalog")),
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
                    match self.database.update_image_data(entry) {
                        Ok(()) => {},
                        Err(err) => return Err(anyhow!(err)),
                    }
                };
                self.count_selected();
                Ok(())
            },
            None => Err(anyhow!("empty catalog")),
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
                    match self.database.update_image_data(entry) {
                        Ok(()) => {},
                        Err(err) => return Err(anyhow!(err)),
                    }
                };
                self.count_selected();
                Ok(())
            },
            None => Err(anyhow!("empty catalog")),
        }
    }

    pub fn count_selected(&mut self) {
        self.max_selected = self.picture_entries.clone().iter().filter(|entry| entry.selected).count()
    }

    pub fn sort_by(&mut self, order: Order) {
        if let Some(entry) = self.current_entry() {
            let original_file_path = entry.original_file_path();
            match order {
                Order::Colors => self.picture_entries.sort_by(|a, b| { a.colors.cmp(&b.colors) }),
                Order::Date => self.picture_entries.sort_by(|a, b| { a.modified_time.cmp(&b.modified_time) }),
                Order::Name => self.picture_entries.sort_by(|a, b| { a.original_file_path().cmp(&b.original_file_path()) }),
                Order::Size => self.picture_entries.sort_by(|a, b| { a.file_size.cmp(&b.file_size)} ),
                Order::Value => self.picture_entries.sort_by(|a, b|  { a.cmp_rank(&b) }),
                Order::Label => self.picture_entries.sort_by(|a, b| { a.cmp_label(&b) }),
                Order::Palette => self.picture_entries.sort_by(|a, b| { a.palette.cmp(&b.palette) }),
                Order::Random => self.picture_entries.shuffle(&mut thread_rng()),
            };
            self.order = Some(order);
            if let Some(index) = self.picture_entries.iter().position(|entry| entry.original_file_path() == original_file_path) {
                self.move_to_index(index)
            } else {
                panic!("couldn't find entry with original file name= {}", original_file_path)
            }
        }
    }

    pub fn move_to_random_index(&mut self) {
        let index = thread_rng().gen_range(0..self.length());
        if self.can_move_to_index(index) {
            self.move_to_index(index)
        }
    }

    pub fn move_to_index(&mut self, index: usize) {
        if index != self.index {
            let old_page_index = self.page_index();
            self.index = index;
            self.page_changed = self.page_index() != old_page_index
        }
    }

    pub fn move_to_first(&mut self) {
        self.move_to_index(0)
    }

    pub fn move_to_last(&mut self) {
        self.move_to_index(self.last())
    }

    pub fn move_next_page(&mut self) {
        let candidate_index = self.page_index() + self.page_length();
        self.move_to_index( if candidate_index < self.length() { candidate_index } else { 0 });
    }

    pub fn move_towards(&mut self, direction: Direction) {
        if self.can_move_towards(direction.clone()) {
            let mut index = self.index;
            match direction {
                Direction::Right => if index + 1 < self.length() { index += 1 },
                Direction::Left => if index > 0 { index -= 1 },
                Direction::Down => if index + self.page_size < self.length() { index += self.page_size } else { index = 0 },
                Direction::Up => {
                    if index >= self.page_size {
                        index -= self.page_size
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

    pub fn move_prev_page(&mut self) {
        let index = if self.page_index() >= self.page_length() {
            self.page_index() - self.page_length()
        } else {
            self.page_index_of(self.length()-1)
        };
        self.move_to_index(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use crate::rank::Rank;
    use crate::picture_entry::make_picture_entry;
    use std::time::SystemTime;
    use chrono::DateTime;

    const PGM: &str = "gsr";

    fn my_checked_args(command_line: Vec<&str>) -> Result<Args> {
        let result = Args::try_parse_from(command_line.iter());
        let mut args = result.unwrap();
        args.checked_args()
    }

    fn my_entries() -> Vec<PictureEntry> {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let day_b: SystemTime = DateTime::parse_from_rfc2822("Sat, 1 Jul 2023 10:52:37 GMT").unwrap().into();
        let day_c: SystemTime = DateTime::parse_from_rfc2822("Mon, 1 Jan 2024 10:52:37 GMT").unwrap().into();
        let day_d: SystemTime = DateTime::parse_from_rfc2822("Mon, 1 Jan 2024 11:52:37 GMT").unwrap().into();
        vec!(
            make_picture_entry(String::from("testdata/foo.jpeg"), 100, 5, day_d, Rank::NoStar, None, Some(String::from("foo")), false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/bar.jpeg"), 1000, 15, day_b, Rank::ThreeStars, None, None, false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/qux.jpeg"), 10, 25, day_c, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None, false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/bub.jpeg"), 100, 25, day_a, Rank::OneStar, None, Some(String::from("xanadoo")),false, false, HashSet::new()))
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
            make_picture_entry(String::from("testdata/joe.jpeg"), 100, 5, day_a, Rank::NoStar, None, Some(String::from("foo")),false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/gus.jpeg"), 1000, 15, day_a, Rank::ThreeStars, None, None, false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/zoo.jpeg"), 10, 25, day_a, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None, false, false, HashSet::new())];
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
        catalog.toggle_page_limit();
        assert_eq!(true, catalog.page_limit_on);
        catalog.set_page_size(2);
        catalog.move_to_index(0);
        catalog.move_towards(Direction::Right);
        assert_eq!(1, catalog.index().unwrap());
        assert_eq!(4, catalog.page_length());
        assert_eq!(true, catalog.can_move_towards(Direction::Down));
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
        assert_eq!(false, catalog.page_limit_on);
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
    fn finding_a_picture_entry_by_input_pattern() {
        let mut example = my_entries();
        let mut catalog = Catalog::new();
        catalog.add_picture_entries(&mut example);
        catalog.sort_by(Order::Size);
        catalog.move_to_index(0);
        assert_eq!(String::from("qux.jpeg"),catalog.current_entry().unwrap().original_file_name());
        let index = catalog.index_input_pattern("fo");
        assert_eq!(true, index.is_some());
        catalog.move_to_index(index.unwrap());
        assert_eq!(String::from("foo.jpeg"), catalog.current_entry().unwrap().original_file_name());
        assert_eq!(None, catalog.index_input_pattern("qa"));
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
        catalog.delete();
        assert_eq!(true, catalog.current_entry().unwrap().deleted);
        let _ = catalog.toggle_select();
        assert_eq!(false, catalog.current_entry().unwrap().selected);
        catalog.delete();
        assert_eq!(false, catalog.current_entry().unwrap().deleted);
        let _ = catalog.toggle_select();
        assert_eq!(true, catalog.current_entry().unwrap().selected);
    }

    #[test] 
    fn adding_entries_from_a_directory() {
        let args = my_checked_args(vec![PGM, "testdata"]);
        let catalog = Catalog::init_catalog(&args.unwrap()).expect("failed to create catalog");
        assert_eq!(10, catalog.length())
    }

    #[test] 
    fn adding_entries_from_a_directory_with_pattern_option() {
        let args = my_checked_args(vec![PGM, "testdata/nature", "--pattern", "or" ]);
        let catalog = Catalog::init_catalog(&args.unwrap()).expect("failed to create catalog");
        println!("{:?}", catalog.picture_entries().iter().map(|e| e.file_path.clone()).collect::<Vec<_>>());
        assert_eq!(1, catalog.length());
        assert_eq!(String::from("labrador.jpg"), catalog.picture_entries[0].original_file_name());
    }
}

