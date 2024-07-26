use gtk::{Align, ApplicationWindow, gdk, Picture, ScrolledWindow};
use gtk::gdk::Key;
use crate::gdk::Display;
use crate::direction::Direction;
use crate::Args;
use crate::Catalog;
use crate::catalog::InputKind;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::glib::clone;

struct Gui {
    application_window: gtk::ApplicationWindow,
    view_scrolled_window: gtk::ScrolledWindow,
    single_view_picture: gtk::Picture,
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

    let view_scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .name("view")
        .build();

    let picture = Picture::new();
    view_scrolled_window.set_child(Some(&picture));
    application_window.set_child(Some(&view_scrolled_window));

    let gui = Gui {
        application_window: application_window,
        view_scrolled_window: view_scrolled_window,
        single_view_picture: picture,
    };
    let gui_rc = Rc::new(RefCell::new(gui));

    let evk = gtk::EventControllerKey::new();
    evk.connect_key_pressed(clone!(@strong catalog_rc, @strong gui_rc => move |_, key, _, _| {
        process_key(&catalog_rc, &gui_rc, key) 
    }));
    if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
        if let Ok(gui) = gui_rc.try_borrow() {
            gui.application_window.add_controller(evk);
            catalog.refresh();
            refresh_single_view_picture(&gui, &catalog);
            gui.application_window.present()
        }
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
                let mut refresh: bool = true;
                if catalog.input_on() {
                    refresh = false;
                    match key_name.as_str() {
                        "Escape" => catalog.cancel_input(),
                        "Return" => {
                            catalog.confirm_input();
                            refresh = true
                        },
                        "BackSpace" => catalog.del_input_char(),
                        _ => {
                            if let Some(ch) = key.to_unicode() {
                                catalog.add_input_char(ch)
                            }
                        },
                    };
                    set_title(&gui, &catalog);
                } else {
                    match key_name.as_str() {
                        "c" => catalog.copy_label(),
                        "D" => catalog.delete(),
                        "e" => catalog.toggle_expand(),
                        "f" => catalog.toggle_full_size(),
                        "g" => catalog.begin_input(InputKind::IndexInput),
                        "n" => {
                            catalog.move_next_page();
                        },
                        "p" => {
                            catalog.move_prev_page();
                        },
                        "q" => gui.application_window.close(),
                        "s" => catalog.begin_input(InputKind::SearchInput),
                        "u" => { let _ = catalog.unselect_page(); },
                        "U" => { let _ = catalog.unselect_all(); },
                        "z" => catalog.move_to_first(),
                        "Z" => catalog.move_to_last(),
                        
                        "comma" => {
                                let _ = catalog.select();
                                catalog.count_selected()
                        },
                        "plus" => {
                            catalog.paste_label();
                        },
                        "minus" => { 
                            let _ = catalog.unlabel();
                        },
                        "slash" => catalog.begin_input(InputKind::LabelInput),
                        "Right" => {
                            refresh = !catalog.full_size_on();
                            arrow_command(Direction::Right, &gui, &mut catalog)
                        },
                        "Left" => {
                            refresh = !catalog.full_size_on();
                            arrow_command(Direction::Left, &gui, &mut catalog)
                        },
                        "Down" => {
                            refresh = !catalog.full_size_on();
                            arrow_command(Direction::Down, &gui, &mut catalog)
                        },
                        "Up" => {
                            refresh = !catalog.full_size_on();
                            arrow_command(Direction::Up, &gui, &mut catalog)
                        },
                        _ => { } ,
                    }
                }
                if refresh { refresh_single_view_picture(&gui, &catalog) }
            };
        }
    };
    gtk::Inhibit(false)
}

pub fn refresh_single_view_picture(gui: &Gui, catalog: &Catalog) {
    let picture = &gui.single_view_picture;
    if catalog.page_changed() {
        let entry = catalog.current_entry().unwrap();
        let opacity = if entry.deleted { 0.25 }
        else if entry.selected { 0.50 } else { 1.0 };
        if catalog.expand_on() {
            picture.set_valign(Align::Fill);
            picture.set_halign(Align::Fill);
        } else {
            picture.set_valign(Align::Center);
            picture.set_halign(Align::Center);
        };
        picture.set_opacity(opacity);
        picture.set_can_shrink(!catalog.full_size_on());
        picture.set_filename(Some(entry.original_file_path()));
        set_title(gui, catalog);
    }
}

pub fn set_title(gui: &Gui, catalog: &Catalog) {
    gui.application_window.set_title(Some(&catalog.title_display()))
}

pub fn arrow_command(direction: Direction, gui: &Gui, catalog: &Catalog) {
    if catalog.full_size_on() {
        let step: f64 = 100.0;
        let (picture_adjustment, step) = match direction {
            Direction::Right => (gui.view_scrolled_window.hadjustment(), step),
            Direction::Left  => (gui.view_scrolled_window.hadjustment(), -step),
            Direction::Down  => (gui.view_scrolled_window.vadjustment(), step),
            Direction::Up    => (gui.view_scrolled_window.vadjustment(), -step),
        };
        picture_adjustment.set_value(picture_adjustment.value() + step)
    }
}

