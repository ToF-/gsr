use gtk::{ApplicationWindow, gdk, Picture};
use gtk::gdk::Key;
use crate::gdk::Display;
use crate::Args;
use crate::Catalog;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::glib::clone;

struct Gui {
    application_window: gtk::ApplicationWindow,
    picture: gtk::Picture,
}

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

    let gui = Gui {
        application_window: application_window,
        picture: picture,
    };
    let gui_rc = Rc::new(RefCell::new(gui));

    let evk = gtk::EventControllerKey::new();
    evk.connect_key_pressed(clone!(@strong catalog_rc, @strong gui_rc => move |_, key, _, _| {
        process_key(&catalog_rc, &gui_rc, key) 
    }));
    if let Ok(gui) = gui_rc.try_borrow() {
        gui.application_window.add_controller(evk);
        gui.application_window.present()
    };
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

pub fn process_key(catalog_rc: &Rc<RefCell<Catalog>>, gui_rc: &Rc<RefCell<Gui>>, key: Key) -> gtk::Inhibit {
    if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
        if let Ok(gui) = gui_rc.try_borrow() {
            if let Some(key_name) = key.name() {
                match key_name.as_str() {
                    "n" => {
                        catalog.move_next_page();
                        let entry = catalog.current_entry().unwrap();
                        gui.picture.set_filename(Some(entry.original_file_path()))
                    },
                    "p" => {
                        catalog.move_prev_page();
                        let entry = catalog.current_entry().unwrap();
                        gui.picture.set_filename(Some(entry.original_file_path()))
                    },
                    "q" => gui.application_window.close(),
                    _ => { } ,
                }
            }
        }
    };
    gtk::Inhibit(false)
}
