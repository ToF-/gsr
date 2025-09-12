use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};
use crate::args::Args;
use crate::comment::Comment;
use crate::database::Database;
use crate::display::title_display;
use crate::editor::{Editor};
use crate::loader::load_picture_entries_from_source;
use crate::navigator::Navigator;
use crate::order::Order;
use crate::path::check_path;
use crate::path::file_name;
use crate::path::file_path_directory;
use crate::picture_entry::{PictureEntries, PictureEntry};
use crate::picture_io::{append_to_extract_file, copy_file_to_target_directory, delete_file, check_or_create_thumbnail_file};
use crate::rank::Rank;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::cmp::Ordering::{Less, Greater, Equal};
use std::collections::HashMap;
use std::collections::HashSet;
use std::process::exit;

pub type Coords = (usize, usize);

#[derive(Debug)]
pub struct Catalog {
    args: Option<Args>,
    copied_label: Option<String>,
    database: Database,
    discarded: Vec<usize>,
    expand_on: bool,
    full_size_on: bool,
    last_comment: Option<Comment>,
    navigator: Navigator,
    order: Option<Order>,
    palette_on: bool,
    picture_entries: Vec<PictureEntry>,
    previous_order: Option<Order>,
    selected_count: usize,
    pub tags: HashSet<String>,
}

impl Catalog {

    // creation

    pub fn new() -> Self {
        Catalog {
            navigator: Navigator::new(),
            picture_entries : Vec::new(),
            copied_label: None,
            last_comment: None,
            palette_on: false,
            full_size_on: false,
            expand_on: false,
            order: Some(Order::Random),
            selected_count: 0,
            previous_order: Some(Order::Random),
            args: None,
            discarded: Vec::new(),
            database: match Database::initialize(false) {
                Ok(database) => database,
                Err(err) => {
                    eprintln!("{}", err);
                    exit(1);
                }
            },
            tags: HashSet::new(),
        }
    }


    pub fn exit(&mut self) {
        self.navigator.exit()
    }

    pub fn init_catalog(args: &Args) -> Result<Self> {
        println!("initializing…");
        let mut catalog = Self::new();
        catalog.args = Some(args.clone());
        catalog.set_page_size(catalog.args.clone().unwrap().grid.unwrap());
        let picture_entries = load_picture_entries_from_source(&mut catalog.database, args);
        Catalog::set_picture_entries(&mut catalog, picture_entries)?;
        catalog.count_selected();
        if catalog.navigator().length() == 0 {
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
        self.navigator.set_length(self.picture_entries.len());

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

    pub fn delete_picture_entry(&self, picture_entry: &PictureEntry) -> Result<()> {
        println!("deleting {}", picture_entry.original_file_path());
        self.database.delete_picture(&picture_entry.original_file_path())
            .and_then(|_| {
                delete_file(&picture_entry.original_file_path())
                    .and_then(|_| {
                        delete_file(&picture_entry.thumbnail_file_path())
                    })
            })
    }

    pub fn delete_files(&self) -> Result<()> {
        for entry in self.picture_entries.clone().iter()
            .filter(|entry| entry.deleted) {
                match self.delete_picture_entry(entry) {
                    Ok(()) => {},
                    Err(err) => {
                        eprintln!("{}", err)
                    },
                }
            }
        Ok(())
    }

    fn redirect_picture_entry_files(&self, picture_entry: &PictureEntry, path: &Path) -> Result<()> {
        let mut new_picture_file_path_buf:PathBuf = PathBuf::from(path);
        let mut new_thumb_file_path_buf:PathBuf = PathBuf::from(path);
        new_picture_file_path_buf.push(file_name(&picture_entry.original_file_path()));
        new_thumb_file_path_buf.push(file_name(&picture_entry.thumbnail_file_path()));
        let new_picture_file_path: String = new_picture_file_path_buf.display().to_string();
        let new_thumb_file_path: String = new_thumb_file_path_buf.display().to_string();
        if  new_picture_file_path != picture_entry.original_file_path() {
            println!("redirecting {} to {}", picture_entry.original_file_path(), new_picture_file_path);
            println!("redirecting {} to {}", picture_entry.thumbnail_file_path(), new_thumb_file_path);
            let result = self.database.insert_new_picture_with_file_path(picture_entry, &new_picture_file_path)
                .and_then(|_| copy_file_to_target_directory(&picture_entry.original_file_path(), &path.display().to_string()))
                .and_then(|_| copy_file_to_target_directory(&picture_entry.thumbnail_file_path(),&path.display().to_string()))
                .and_then(|_| self.delete_picture_entry(picture_entry));
            match result {
                Ok(_) => { },
                Err(err) => {
                    eprintln!("{}", err)
                }
            }
        };
        Ok(())
    }

    pub fn redirect_files(&self) -> Result<()> {
        match self.args.clone().unwrap().redirect {
            Some(target) => {
                for entry in self.picture_entries.clone().iter()
                    .filter(|entry| entry.image_data.selected && !entry.deleted && entry.label().is_some()) {
                        let new_directory: String = if target.clone().ends_with("/") {
                            target.clone() + &entry.label().unwrap()
                        } else {
                            target.clone() + "/" + &entry.label().unwrap()
                        };
                        match check_path(&new_directory, true) {
                            Ok(path) => {
                                let _ = self.redirect_picture_entry_files(entry, &path);
                            }
                            Err(err) => { 
                                eprintln!("{}", err)
                            }
                        }
                    };
                Ok(())
            },
            None => Ok(()),
        }
    }
    // queries

    pub fn navigator(&self) -> &Navigator {
        &self.navigator
    }
    
    pub fn mut_navigator(&mut self) -> &mut Navigator {
        &mut self.navigator
    }
    
    pub fn copied_label(&self) -> Option<String> {
        self.copied_label.clone()
    }
    pub fn picture_entries(&self) -> &Vec<PictureEntry> {
        &self.picture_entries
    }
    
    pub fn args(&self) -> Option<Args> {
        self.args.clone()
    }

    pub fn selected_count(&self) -> usize {
        self.selected_count
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

    pub fn expand_on(&self) -> bool {
        self.expand_on
    }

    pub fn last(&self) -> usize {
        self.navigator.last()
    }

    pub fn page_size(&self) -> usize {
        self.navigator.page_size()
    }

    pub fn page_length(&self) -> usize {
        self.navigator.page_length()
    }

    pub fn palette_on(&self) -> bool {
        self.palette_on
    }

    pub fn full_size_on(&self) -> bool {
        self.full_size_on
    }

    pub fn index(&self) -> Option<usize> {
        self.navigator.index()
    }

    pub fn position_from_index(&self, index: usize) -> Coords {
        self.navigator.position_from_index(index)

    }

    pub fn index_from_position(&self, coords: Coords) -> Option<usize> {
        self.navigator.clone().index_from_position(coords)
    }

    pub fn entry_at_index(&self, index: usize) -> Option<&PictureEntry> {
        if index < self.picture_entries.len() {
            Some(&self.picture_entries[index])
        } else {
            None
        }
    }

    pub fn current_entry(&self) -> Option<&PictureEntry> {
        self.navigator.index().and_then(|index| { self.entry_at_index(index) } )
    }

    pub fn find_index_input_pattern(&mut self, pattern: &str) -> Option<usize> {
        self.picture_entries.iter().position(|entry|
            entry.original_file_path().contains(pattern))
    }

    pub fn find_index_label_search(&mut self, pattern: &str) -> Option<usize> {
        self.picture_entries.iter().position(|entry|
            if let Some(label) = entry.label() {
                label.contains(pattern)
            } else {
                false
            })
    }

    pub fn page_changed(&self) -> bool {
        self.navigator.page_changed()
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
            if let Some(extract_file_path) = &args.list_extract {
                match append_to_extract_file(&file_path, extract_file_path) {
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
        match self.navigator.index() {
            Some(index) => {
                self.picture_entries[index] = picture_entry.clone();
                self.database.update_picture_entry(&picture_entry)
            },
            None => Ok(())
        }
    }

    pub fn set_picture_entries(&mut self, picture_entries_result: Result<PictureEntries>) -> Result<()> {
        match picture_entries_result {
            Ok(picture_entries) => {
                self.picture_entries = picture_entries;
                self.navigator.set_length(self.picture_entries.len());
                Ok(())
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn cover_current_entry(&mut self) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.image_data.cover = true;
                let dir_path = file_path_directory(&picture_entry.file_path);
                let file_name = file_name(&picture_entry.file_path);
                let rank = picture_entry.image_data.rank;
                match self.database.insert_or_update_cover(&dir_path, &file_name, rank) {
                    Ok(()) => {
                        self.last_comment = Some(Comment::Cover);
                        self.set_current_picture_entry(new_picture_entry)
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            None => Ok(())
        }
    }
    pub fn uncover_current_entry(&mut self) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.image_data.cover = false;
                let dir_path = file_path_directory(&picture_entry.file_path);
                let file_name = file_name(&picture_entry.file_path);
                match self.database.delete_cover(&dir_path, &file_name) {
                    Ok(()) => {
                        self.last_comment = Some(Comment::Uncover);
                        self.set_current_picture_entry(new_picture_entry)
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            None => Ok(())
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

    pub fn paste_label_current_entry(&mut self) -> Result<()> {
        match self.copied_label() {
            Some(label) => match self.current_entry() {
                Some(picture_entry) => {
                    let mut new_picture_entry = picture_entry.clone();
                    new_picture_entry.set_label(&label);
                    match self.set_current_picture_entry(new_picture_entry) {
                        Ok(()) => {
                            self.last_comment = Some(Comment::Label { label } );
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

    pub fn untag_current_entry(&mut self, tag: &str) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.delete_tag(tag);
                match self.set_current_picture_entry(new_picture_entry) {
                    Ok(()) => {
                        self.tags.insert(tag.to_string());
                        self.last_comment = Some(Comment::DeleteTag { label: tag.to_string() });
                        Ok(())
                    },
                    Err(err) => Err(anyhow!(err)),
                } 
            },
            None => Ok(()),
        }
    }

    pub fn toggle_select_current_entry(&mut self) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.image_data.selected = !new_picture_entry.image_data.selected;
                match self.set_current_picture_entry(new_picture_entry) {
                    Ok(()) => {
                        self.last_comment = Some(Comment::ToggleSelect);
                        Ok(())
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            None => Ok(()),
        }
    }

    pub fn toggle_delete_current_entry(&mut self) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.deleted = !new_picture_entry.deleted;
                match self.set_current_picture_entry(new_picture_entry) {
                    Ok(()) => {
                        self.last_comment = Some(Comment::ToggleDelete);
                        Ok(())
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            None => Ok(()),
        }
    }

    pub fn begin_sort_selection(&mut self) {
        self.previous_order = self.order;
        self.order = None
    }

    pub fn cancel_sort_selection(&mut self) {
        self.order = self.previous_order
    }

    pub fn toggle_expand(&mut self) {
        self.expand_on = !self.expand_on;
        self.navigator.change_page()
    }

    pub fn refresh(&mut self) {
        self.navigator.change_page();
    }

    pub fn set_page_size(&mut self, page_size: usize) {
        self.navigator.set_page_size(page_size)
    }

    pub fn set_new_page_size(&mut self, page_size: usize) {
        self.navigator.set_new_page_size(page_size)
    }

    pub fn move_to_input_pattern(&mut self, pattern: &str) {
        if let Some(index) = self.find_index_input_pattern(pattern) {
            if self.navigator.can_move_to_index(index) {
                self.navigator.move_to_index(index)
            }
        }
    }

    pub fn move_to_label_pattern(&mut self, pattern: &str) {
        println!("move_to_label_pattern({})", pattern);
        if let Some(index) = self.find_index_label_search(pattern) {
            if self.navigator.can_move_to_index(index) {
                self.navigator.move_to_index(index)
            }
        }
    }

    pub fn start_set(&mut self) {
        self.navigator.start_set();
        println!("{}…", self.navigator.start_index().unwrap())
    }

    pub fn cancel_set(&mut self) {
        if self.current_entry().is_some() {
            self.navigator = self.navigator.cancel_set()
        }
    }

    pub fn rank_current_entry(&mut self, rank: Rank) -> Result<()> {
        match self.current_entry() {
            Some(picture_entry) => {
                let mut new_picture_entry = picture_entry.clone();
                new_picture_entry.set_rank(rank);
                match self.set_current_picture_entry(new_picture_entry) {
                    Ok(()) => {
                        self.last_comment = Some(Comment::Rank { rank });
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
            self.navigator = self.navigator.set_index(i);
            if let Some(entry) = self.current_entry() {
                match entry.label() {
                    Some(_) => {},
                    None => match self.label_current_entry(label) {
                        Ok(()) => {},
                        Err(err) => return Err(err),
                    }
                }
            }
        }
        Ok(())
    }
        
    pub fn print_labels_all(&mut self) -> Result<()> {
        let mut tags:HashMap<String,usize> = HashMap::new();

        for i in 0..self.picture_entries.len() {
            self.navigator = self.navigator.set_index(i);
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
            Some(Comment::Unlabel) => self.unlabel_current_entry(),
            Some(Comment::AddTag { label}) => self.tag_current_entry(&label),
            Some(Comment::DeleteTag { label}) => self.untag_current_entry(&label),
            Some(Comment::Rank { rank }) => self.rank_current_entry(rank),
            Some(Comment::Cover) => self.cover_current_entry(),
            Some(Comment::Uncover) => self.uncover_current_entry(),
            Some(Comment::ToggleSelect) => self.toggle_select_current_entry(),
            Some(Comment::ToggleDelete) => self.toggle_delete_current_entry(),
        }
    }

    pub fn set_selected_labels_with_input(&mut self, label: &str) -> Result<()> {
        for index in 0..self.picture_entries.len() {
            let entry = &mut self.picture_entries[index];
            if entry.image_data.selected {
                entry.set_label(label);
                match self.database.update_picture_entry(entry) {
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

    pub fn end_repeat_last_comment(&mut self) -> Result<()> {
        match self.navigator.index() {
            Some(index) => {
                match self.navigator.start_index() {
                    None => self.repeat_last_comment(),
                    Some(other) => {
                        let (start,end) = if other <= index { (other,index) } else { (index,other) };
                        match &self.last_comment {
                            None => {},
                            Some(comment) => { println!("[{}…{}] {}", start, end, comment); },
                        };
                        for i in start..end+1 {
                            let entry: &mut PictureEntry = &mut self.picture_entries[i];
                            match &self.last_comment {
                                None => {},
                                Some(Comment::Label { label }) => entry.set_label(label),
                                Some(Comment::Unlabel) => entry.unlabel(),
                                Some(Comment::AddTag { label}) => entry.add_tag(label),
                                Some(Comment::DeleteTag { label}) => entry.delete_tag(label),
                                Some(Comment::Rank { rank }) => entry.set_rank(*rank),
                                Some(Comment::ToggleSelect) => { entry.image_data.selected = !entry.image_data.selected }
                                Some(Comment::ToggleDelete) => { entry.deleted = !entry.deleted },
                                Some(Comment::Cover) => { entry.image_data.cover = true },
                                Some(Comment::Uncover) => { entry.image_data.cover = false },
                            };
                            if self.last_comment.is_some() {
                                match self.database.update_picture_entry(&entry.clone()) {
                                    Ok(()) => {},
                                    Err(err) => return Err(anyhow!(err)),
                                }
                            };
                        };
                        self.navigator = self.navigator.cancel_set();
                        Ok(())
                    },
                }
            },
            None => Err(anyhow!("empty catalog")),
        }
    }

    pub fn unselect_page(&mut self) -> Result<()> {
        match self.navigator.index() {
            Some(_) => {
                let start = self.navigator.page_index();
                let end = start + self.page_length();
                for i in start..end {
                    let entry: &mut PictureEntry = &mut self.picture_entries[i];
                    entry.image_data.selected = false;
                    match self.database.update_picture_entry(entry) {
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
        match self.navigator.index() {
            Some(_) => {
                let start = 0;
                let end = self.navigator.length();
                for i in start..end {
                    let entry: &mut PictureEntry = &mut self.picture_entries[i];
                    entry.image_data.selected = false;
                    match self.database.update_picture_entry(entry) {
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
        self.selected_count = self.picture_entries.clone().iter().filter(|entry| entry.image_data.selected).count()
    }

    pub fn sort_by(&mut self, order: Order) {
        if let Some(entry) = self.current_entry() {
            let original_file_path = entry.original_file_path();
            match order {
                Order::Colors => self.picture_entries.sort_by(|a, b| { a.image_data.colors.cmp(&b.image_data.colors) }),
                Order::Date => self.picture_entries.sort_by(|a, b| { a.modified_time.cmp(&b.modified_time) }),
                Order::Name => self.picture_entries.sort_by(|a, b| { a.original_file_path().cmp(&b.original_file_path()) }),
                Order::Size => self.picture_entries.sort_by(|a, b| { a.file_size.cmp(&b.file_size)} ),
                Order::Value => self.picture_entries.sort_by(|a, b|  { a.cmp_rank(b) }),
                Order::Label => self.picture_entries.sort_by(|a, b| { a.cmp_label(b) }),
                Order::Palette => self.picture_entries.sort_by(|a, b| { a.image_data.palette.cmp(&b.image_data.palette) }),
                Order::Random => self.picture_entries.shuffle(&mut thread_rng()),
            };
            self.order = Some(order);
            if let Some(index) = self.picture_entries.iter().position(|entry| entry.original_file_path() == original_file_path) {
                self.navigator.move_to_index(index)
            } else {
                panic!("couldn't find entry with original file name= {}", original_file_path)
            }
        }
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
            make_picture_entry(String::from("testdata/foo.jpeg"), 100, 5, day_d, Rank::NoStar, None, Some(String::from("foo")), false, false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/bar.jpeg"), 1000, 15, day_b, Rank::ThreeStars, None, None, false, false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/qux.jpeg"), 10, 25, day_c, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None, false, false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/bub.jpeg"), 100, 25, day_a, Rank::OneStar, None, Some(String::from("xanadoo")),false, false, false, HashSet::new()))
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
            make_picture_entry(String::from("testdata/joe.jpeg"), 100, 5, day_a, Rank::NoStar, None, Some(String::from("foo")),false, false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/gus.jpeg"), 1000, 15, day_a, Rank::ThreeStars, None, None, false, false, false, HashSet::new()),
            make_picture_entry(String::from("testdata/zoo.jpeg"), 10, 25, day_a, Rank::TwoStars, Some([1,1,1,1,1,1,1,1,1]), None, false, false, false, HashSet::new())];
        let mut catalog = my_catalog();
        catalog.add_picture_entries(&mut other_entries);
        catalog
    }

    #[test]
    fn at_creation_length_is_0() {
        let catalog = Catalog::new();
        assert_eq!(catalog.navigator().length(), 0);
        assert_eq!(true, catalog.current_entry().is_none());
    }

    #[test]
    fn after_adding_entries_length_is_updated() {
        let catalog = my_catalog();
        assert_eq!(catalog.navigator().length(), 4);
    }


    #[test]
    fn sorting_catalog_by_different_criteria() {
        let mut catalog = my_catalog();
        catalog.sort_by(Order::Size);
        catalog.mut_navigator().move_to_index(0);
        assert_eq!(String::from("qux.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Date);
        catalog.mut_navigator().move_to_index(0);
        assert_eq!(String::from("bub.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Name);
        catalog.mut_navigator().move_to_index(0);
        assert_eq!(String::from("bar.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Colors);
        catalog.mut_navigator().move_to_index(0);
        assert_eq!(String::from("foo.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Value);
        catalog.mut_navigator().move_to_index(3);
        assert_eq!(String::from("foo.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Label);
        catalog.mut_navigator().move_to_index(0);
        assert_eq!(String::from("foo.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.mut_navigator().move_to_index(1);
        assert_eq!(String::from("bub.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
        catalog.sort_by(Order::Palette);
        catalog.mut_navigator().move_to_index(3);
        assert_eq!(String::from("qux.jpeg"),
            catalog.current_entry().unwrap().original_file_name());
    }
//    }
//    #[test]
//    fn finding_a_picture_entry_by_input_pattern() {
//        let mut example = my_entries();
//        let mut catalog = Catalog::new();
//        catalog.add_picture_entries(&mut example);
//        catalog.sort_by(Order::Size);
//        catalog.mut_navigator().move_to_index(0);
//        assert_eq!(String::from("qux.jpeg"),catalog.current_entry().unwrap().original_file_name());
//        let index = catalog.index_input_pattern("fo");
//        assert_eq!(true, index.is_some());
//        catalog.navigator.move_to_index(index.unwrap());
//        assert_eq!(String::from("foo.jpeg"), catalog.current_entry().unwrap().original_file_name());
//        assert_eq!(None, catalog.index_input_pattern("qa"));
//    }
//
//    #[test]
//    fn state_indicators() {
//        let mut catalog = Catalog::new();
//        assert_eq!(false, catalog.palette_on());
//        assert_eq!(false, catalog.full_size_on());
//        catalog.toggle_palette();
//        catalog.toggle_full_size();
//        assert_eq!(true, catalog.palette_on());
//        assert_eq!(true, catalog.full_size_on());
//
//    }
//
//
//
//    #[test] 
//    fn adding_entries_from_a_directory() {
//        let args = my_checked_args(vec![PGM, "testdata"]);
//        let catalog = Catalog::init_catalog(&args.unwrap()).expect("failed to create catalog");
//        assert_eq!(10, catalog.length())
//    }
//
//    #[test] 
//    fn adding_entries_from_a_directory_with_pattern_option() {
//        let args = my_checked_args(vec![PGM, "testdata/nature", "--pattern", "or" ]);
//        let catalog = Catalog::init_catalog(&args.unwrap()).expect("failed to create catalog");
//        println!("{:?}", catalog.picture_entries().iter().map(|e| e.file_path.clone()).collect::<Vec<_>>());
//        assert_eq!(1, catalog.length());
//        assert_eq!(String::from("labrador.jpg"), catalog.picture_entries[0].original_file_name());
//    }
}

