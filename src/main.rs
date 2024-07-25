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
use glib::{clone};
use crate::gui::{build_gui, startup_gui};
use gtk::prelude::*;
use gtk::{self, Application, gdk, glib};
use std::process::exit;

fn main() {
    let result = Args::parse().checked_args();
    match result {
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        },
        Ok(args) => {
            let application = Application::builder()
                .application_id("org.example.gallsh")
                .build();

            application.connect_startup(|application| {
                startup_gui(application);
            }); 

            // clone! passes a strong reference to a variable in the closure that activates the application
            // move converts any variables captured by reference or mutable reference to variables captured by value.
            application.connect_activate(clone!(@strong args => move |application: &gtk::Application| {
                build_gui(application, &args)
            }));

            let empty: Vec<String> = vec![];
            application.run_with_args(&empty);
        }
    }
}

