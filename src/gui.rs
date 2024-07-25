use gtk::{ApplicationWindow, gdk, Picture};
use crate::gdk::Display;
use crate::Args;
use crate::Catalog;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;


pub fn build_gui(application: &gtk::Application, args: &Args, catalog_rc: &Rc<RefCell<Catalog>>) {
    let width = args.width.unwrap();
    let height = args.height.unwrap();
    let application_window = ApplicationWindow::builder()
        .application(application)
        .title("gsr")
        .default_width(width)
        .default_height(height)
        .build();
    let picture = Picture::new();

    if let Ok(catalog) = catalog_rc.try_borrow() {
        let entry = catalog.current_entry().unwrap();
        picture.set_filename(Some(entry.original_file_path()));
    }
    application_window.set_child(Some(&picture));
    application_window.present();
}

pub fn startup_gui(application: &gtk::Application) {
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data("window { background-color:black;} image { margin:1em ; } label { color:white; }");
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &css_provider,
        1000,
        );
}
