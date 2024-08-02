mod args;
mod catalog;
mod direction;
mod gui;
mod image_data;
mod order;
mod palette;
mod path;
mod picture_entry;
mod picture_io;
mod rank;

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

fn main() {
    let result = Args::parse().checked_args();
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
                    catalog.sort_by(args.order.clone());
                    let catalog_rc = Rc::new(RefCell::new(catalog));
                    let application = Application::builder()
                        .application_id("org.example.gallsh")
                        .build();

                    application.connect_startup(|application| {
                        startup_gui(application);
                    }); 

                    // clone! passes a strong reference to a variable in the closure that activates the application
                    // move converts any variables captured by reference or mutable reference to variables captured by value.
                    application.connect_activate(clone!(@strong args, @strong catalog_rc => move |application: &gtk::Application| {
                        build_gui(application, &args, &catalog_rc)
                    }));

                    let empty: Vec<String> = vec![];
                    application.run_with_args(&empty);
                },
            }
        }
    }
}

