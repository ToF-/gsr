use gtk::{ApplicationWindow, gdk, Picture};
use crate::gdk::Display;
use crate::Args;
use gtk::prelude::*;


pub fn build_gui(application: &gtk::Application, args: &Args) {
    let width = args.width.unwrap();
    let height = args.height.unwrap();
    let application_window = ApplicationWindow::builder()
        .application(application)
        .title("gsr")
        .default_width(width)
        .default_height(height)
        .build();
    let picture = Picture::new();
    picture.set_filename(Some("testdata/nature/flower.jpg"));
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
