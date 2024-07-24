use gtk::{ApplicationWindow};
use gtk::prelude::*;


pub fn build_gui(application: &gtk::Application) {
    let window = ApplicationWindow::builder()
        .application(application)
        .title("gsr")
        .build();
    window.present();
}
