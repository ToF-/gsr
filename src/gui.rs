use gtk::{Align, ApplicationWindow, gdk, Orientation, Picture, ScrolledWindow};
use gtk::cairo::{Context, Format, ImageSurface};
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
    single_view_box: gtk::Box,
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

    let view_box = gtk::Box::new(Orientation::Vertical, 0);
    view_box.set_valign(Align::Fill);
    view_box.set_halign(Align::Fill);
    view_box.set_hexpand(true);
    view_box.set_vexpand(true);
    view_box.set_homogeneous(false);

    let picture = Picture::new();
    picture.set_hexpand(true);
    picture.set_vexpand(true);

    view_box.append(&picture);

    view_scrolled_window.set_child(Some(&view_box));
    application_window.set_child(Some(&view_scrolled_window));

    let gui = Gui {
        application_window: application_window,
        view_scrolled_window: view_scrolled_window,
        single_view_box: view_box,
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
                refresh = if catalog.input_on() {
                    input_mode_process_key(key, &gui, &mut catalog)
                } else {
                    view_mode_process_key(key, &gui, &mut catalog)
                };
                if refresh { refresh_single_view_picture(&gui, &catalog) }
            };
        }
    };
    gtk::Inhibit(false)
}

fn input_mode_process_key(key: Key, gui: &Gui, catalog: &mut Catalog) -> bool {
    let mut refresh: bool = false;
    match key.name() {
        None => refresh = false,
        Some(key_name) => match key_name.as_str() {
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
            }
        },
    };
    set_title(&gui, &catalog);
    refresh
}

fn view_mode_process_key(key: Key, gui: &Gui, catalog: &mut Catalog) -> bool {
    let mut refresh: bool = true;
    match key.name() {
        None => refresh = false,
        Some(key_name) => match key_name.as_str() {
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
            "x" => {
                catalog.toggle_palette();
                refresh = true
            },
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
                arrow_command(Direction::Right, gui, catalog)
            },
            "Left" => {
                refresh = !catalog.full_size_on();
                arrow_command(Direction::Left, gui, catalog)
            },
            "Down" => {
                refresh = !catalog.full_size_on();
                arrow_command(Direction::Down, gui, catalog)
            },
            "Up" => {
                refresh = !catalog.full_size_on();
                arrow_command(Direction::Up, gui, catalog)
            },
            _ => { } ,
        },
    };
    refresh
}

pub fn refresh_single_view_picture(gui: &Gui, catalog: &Catalog) {
    let view_box = &gui.single_view_box;
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
        if let Some(widget) = view_box.last_child() {
            if widget != *picture {
                view_box.remove(&widget)
            }
        }
        if catalog.palette_on() {
            let colors = entry.palette;
            let palette_area = create_palette(colors.clone());
            view_box.insert_child_after(&palette_area, Some(picture));
        }
    }
}

fn create_palette(colors: [u32;9]) -> gtk::DrawingArea {
    let palette_area = gtk::DrawingArea::new();
    palette_area.set_valign(Align::Center);
    palette_area.set_halign(Align::Center);
    palette_area.set_content_width(90);
    palette_area.set_content_height(10);
    palette_area.set_draw_func(move |_, ctx, _, _| {
        draw_palette(ctx, 90, 10, &colors)
    });
    palette_area
}

pub fn draw_palette(ctx: &Context, width: i32, height: i32, colors: &[u32;9]) {
    const COLOR_MAX: f64 = 9.0;
    let square_size: f64 = height as f64;
    let offset: f64 = (width as f64 - (COLOR_MAX as f64 * square_size)) / 2.0;
    let surface = ImageSurface::create(Format::ARgb32, width, height).expect("can't create surface");
    let context = Context::new(&surface).expect("can't create context");
    for (i,w) in colors.iter().enumerate() {
        let r = ((w >> 16) & 255) as u8;
        let g = ((w >> 8) & 255) as u8;
        let b = (w & 255) as u8;
        context.set_source_rgb(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
        let x = i as f64 * square_size;
        context.rectangle(offset + x, 0.0, square_size, square_size);
        context.fill().expect("can't fill rectangle");
    };
    ctx.set_source_surface(&surface, 0.0, 0.0).expect("can't set source surface");
    ctx.paint().expect("can't paint surface")
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

