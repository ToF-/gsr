use std::borrow::BorrowMut;
use anyhow::{anyhow,Result};
use crate::loader::load_picture_entries_from_directory_into_db;
use crate::path::directory;
use crate::display::info;
use crate::loader::check_database_and_files;
use crate::path::copy_all_picture_files;
use clap::Parser;
use crate::args::Args;
use crate::catalog::Catalog;
use glib::{clone};
use crate::gui::{build_gui, startup_gui};
use gtk::prelude::*;
use gtk::{self, Application, glib};
use std::process::exit;
use std::cell::RefCell;
use std::rc::Rc;
use crate::commands::load_shortcuts;
use crate::database::Database;

mod comment;
mod navigator;
mod args;
mod catalog;
mod commands;
mod completion;
mod database;
mod direction;
mod display;
mod editor;
mod gui;
mod loader;
mod image_data;
mod order;
mod palette;
mod path;
mod picture_entry;
mod picture_io;
mod rank;

fn main() {
    // load command shortcuts from the .gallshkey.json file, exit if failed
    let shortcuts = match load_shortcuts() {
        Ok(result) => result,
        Err(err) => {
            println!("{}", err);
            exit(1)
        },
    };

    let main_result = Args::parse().checked_args()
        .and_then(|args| {
            println!("directory: {}", directory(args.clone().directory));
            Database::initialize(args.create_schema)
                .and_then(|mut database| {
                    match database_operations(database.borrow_mut(), &args) { 
                        Ok(_) => {},
                        Err(err) => return Err(anyhow!(err)),
                    };
                    Catalog::init_catalog(&args)
                        .and_then(|mut catalog| {
                            if let Some(ref label) = args.label {
                                match catalog.apply_label_all(label) {
                                    Ok(()) => {},
                                    Err(err) => return Err(anyhow!(err)),
                                }
                            };
                            println!("{:?} entries", catalog.length());
                            if args.update {
                                match catalog.update_files() {
                                    Ok(()) => exit(0),
                                    Err(err) => return Err(anyhow!(err)),
                                }
                            };
                            if args.tags {
                                match catalog.print_labels_all() {
                                    Ok(()) => exit(0),
                                    Err(err) => return Err(anyhow!(err)),
                                }
                            };
                            if args.directories {
                                match catalog.print_directories_all() {
                                    Ok(()) => exit(0),
                                    Err(err) => return Err(anyhow!(err)),
                                }
                            };
                            if args.info {
                                info(&catalog);
                            };
                            if args.deduplicate.is_some() {
                                match catalog.deduplicate_files(&args.deduplicate.unwrap()) {
                                    Ok(()) => return Ok(()),
                                    Err(err) => return Err(anyhow!(err)),
                                }
                            };
                            catalog.sort_by(args.order.clone());
                            let catalog_rc = Rc::new(RefCell::new(catalog));
                            let mut exit: bool = false;
                            while !exit {
                                let application = Application::builder()
                                    .application_id("org.example.gallsh")
                                    .build();

                                application.connect_startup(|application| {
                                    startup_gui(application);
                                }); 
                                // clone! passes a strong reference to a variable in the closure that activates the application
                                // move converts any variables captured by reference or mutable reference to variables captured by value.
                                application.connect_activate(clone!(@strong args, @strong catalog_rc, @strong shortcuts => move |application: &gtk::Application| {
                                    build_gui(application, &args, &catalog_rc, &shortcuts)
                                }));

                                let no_args: Vec<String> = vec![];
                                application.run_with_args(&no_args);
                                // if we exit from the application loop with a new page size, we
                                // are not done and loop again
                                catalog_rc.try_borrow_mut()
                                    .and_then(|mut catalog| {
                                        if catalog.done() {
                                            exit = true
                                        } else {
                                            match catalog.new_page_size() {
                                                Some(size) => { catalog.set_page_size(size) },
                                                None => {},
                                            }
                                        };
                                        Ok(())
                                    });
                            }
                            Ok(())
                        })
                })
        });
    match main_result {
        Ok(()) => exit(0),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

    fn database_operations(database: &mut Database, args: &Args) -> Result<()> {
        if args.check {
            match check_database_and_files(&directory(args.clone().directory), &database) {
                Ok(()) => {},
                Err(err) => return Err(anyhow!(err)),
            }
        };
        if args.purge {
            println!("removing orphan entries from the database…");
            match database.delete_picture_data_where_file_do_not_exists() {
                Ok(count) => {
                    println!("{} pictures removed from the database", count);
                },
                Err(err) => return Err(anyhow!(err)),
            }
        };
        match args.from {
            Some(ref ext_directory) => match args.add {
                Some(ref abs_directory) => match copy_all_picture_files(&ext_directory, &abs_directory) {
                    Ok(()) => {},
                    Err(err) => return Err(anyhow!(err)),
                },
                None => match copy_all_picture_files(&ext_directory, &directory(args.clone().directory)) {
                    Ok(()) => {},
                    Err(err) => return Err(anyhow!(err)),
                }
            },
            None => match args.add {
                Some(ref abs_directory) => match load_picture_entries_from_directory_into_db(database, &abs_directory, false) {
                    Ok(pictures_entries) => {
                        println!("the following pictures have been inserted in the database:");
                        for picture_entry in pictures_entries {
                            println!("{}", picture_entry.file_path)
                        }
                    },
                    Err(err) => return Err(anyhow!(err)),
                }
                None => {},
            }
        };
        Ok(())
    }
