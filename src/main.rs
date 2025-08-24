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

mod args;
mod completion;
mod catalog;
mod database;
mod commands;
mod direction;
mod gui;
mod image_data;
mod order;
mod palette;
mod path;
mod picture_entry;
mod picture_io;
mod rank;

fn main() {
    let result = Args::parse().checked_args();
    let shortcuts = match load_shortcuts() {
        Ok(result) => result,
        Err(err) => {
            println!("{}", err);
            exit(1)
        },
    };
    match result {
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        },
        Ok(args) => {
            match Catalog::init_catalog(&args) {
                Err(err) => {
                    eprintln!("{}", err);
                    exit(1);
                },
                Ok(mut catalog) => {
                    if let Some(ref label) = args.label {
                        match catalog.apply_label_all(label.clone()) {
                            Ok(()) => {},
                            Err(err) => {
                                eprintln!("{}", err);
                                exit(1)
                            }
                        }
                    };
                    println!("{:?} entries", catalog.length());
                    if args.update {
                        match catalog.update_files() {
                            Ok(()) => exit(0),
                            Err(err) => {
                                eprintln!("{}", err);
                                exit(1)
                            },
                        }
                    }
                    if args.tags {
                        match catalog.print_labels_all() {
                            Ok(()) => exit(0),
                            Err(err) => {
                                eprintln!("{}", err);
                                exit(1);
                            },
                        }
                    }
                    if args.directories {
                        match catalog.print_directories_all() {
                            Ok(()) => exit(0),
                            Err(err) => {
                                eprintln!("{}", err);
                                exit(1);
                            },
                        }
                    }
                    if args.info {
                        catalog.info();
                        exit(0);
                    }
                    if args.deduplicate.is_some() {
                        match catalog.deduplicate_files(&args.deduplicate.unwrap()) {
                            Ok(()) => exit(0),
                            Err(err) => {
                                eprintln!("{}", err);
                                exit(1)
                            },
                        }
                    }
                    if !catalog.sample_on() {
                        catalog.sort_by(args.order.clone());
                    }
                    let catalog_rc = Rc::new(RefCell::new(catalog));
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

                    let empty: Vec<String> = vec![];
                    application.run_with_args(&empty);
                },
            }
        }
    }
}

