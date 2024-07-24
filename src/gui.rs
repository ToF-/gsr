use gtk::{ApplicationWindow};
use crate::Args;
use gtk::prelude::*;


pub fn build_gui(application: &gtk::Application, args: &Args) {
    let width = args.width.unwrap();
    let height = args.height.unwrap();
    let window = ApplicationWindow::builder()
        .application(application)
        .title("gsr")
        .default_width(width)
        .default_height(height)
        .build();
    window.present();
}
