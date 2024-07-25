use gtk::{ApplicationWindow, Picture};
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
