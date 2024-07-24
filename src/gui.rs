use crate::picture_io::create_thumbnail;
use crate::picture_entry::PictureEntry;
use thumbnailer::create_thumbnails;
use std::io::{Result, Error, ErrorKind};
use gtk::cairo::{Context, Format, ImageSurface};
use crate::args::Args;
use crate::catalog::Catalog;
use std::cell::{RefCell, RefMut};
use std::path::Path;
use std::rc::Rc;
use gtk::{Align, CssProvider, Grid, Label, Orientation, Picture, ScrolledWindow};
use crate::catalog::Coords;
use gtk::glib::clone;
use gtk::glib::prelude::*;
use gtk::prelude::*;



pub struct Gui {
    pub application_window:   gtk::ApplicationWindow,
    pub stack:                gtk::Stack,
    pub grid_scrolled_window: gtk::ScrolledWindow,
    pub view_scrolled_window: gtk::ScrolledWindow,
    pub picture_grid:       gtk::Grid,
    pub image_view:         gtk::Picture,
}

impl Gui {

    pub fn view_mode(&self) -> bool {
        self.stack.visible_child().unwrap() == self.view_scrolled_window
    }
}

pub fn build_gui(args: &Args, application: &gtk::Application) {
    let width:  i32 = args.width.unwrap();
    let height: i32 = args.height.unwrap();
    let grid_size = args.grid.unwrap();
    match Catalog::init_catalog(args) {
        Ok(catalog) => {
            let catalog_rc = Rc::new(RefCell::new(catalog));
            let gui = create_gui(application, width, height, grid_size, &catalog_rc);
            let gui_rc = Rc::new(RefCell::new(gui));
        },
        Err(err) => {
            eprintln!("{}", err)
        },
    }
}

pub fn create_gui(application: &gtk::Application, width: i32, height: i32, grid_size: usize, catalog_rc: &Rc<RefCell<Catalog>>) -> Gui {
    let application_window = gtk::ApplicationWindow::builder()
        .application(application)
        .default_width(width)
        .default_height(height)
        .build();

    let grid_scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .name("grid")
        .build();

    let view_scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .name("view")
        .build();

    let buttons_css_provider = CssProvider::new();
    buttons_css_provider.load_from_data(
        "
            label {
                color: gray;
                font-size: 12px;
            }
            text-button {
                background-color: black;
            }
        ");

        let view = Grid::new();
        view.set_row_homogeneous(true);
        view.set_column_homogeneous(true);
        view.set_hexpand(true);
        view.set_vexpand(true);
        view_scrolled_window.set_child(Some(&view));

        let stack = gtk::Stack::new();
        stack.set_hexpand(true);
        stack.set_vexpand(true);
        let _ = stack.add_child(&grid_scrolled_window);
        let _ = stack.add_child(&view_scrolled_window);
        stack.set_visible_child(&view_scrolled_window);
        stack.set_visible_child(&grid_scrolled_window);

        application_window.set_child(Some(&stack));

        let image_view = Picture::new();
        let view_gesture = gtk::GestureClick::new();
        view_gesture.set_button(0);
        view_gesture.connect_pressed(clone!(@strong catalog_rc, @strong stack, @strong grid_scrolled_window, @strong application_window => move |_,_, _, _| {
            stack.set_visible_child(&grid_scrolled_window);
        }));

        image_view.add_controller(view_gesture);

        view.attach(&image_view, 0, 0, 1, 1);


        let panel = Grid::new();
        panel.set_hexpand(true);
        panel.set_vexpand(true);
        panel.set_row_homogeneous(true);
        panel.set_column_homogeneous(false);
        let left_button = Label::new(Some("←"));
        let right_button = Label::new(Some("→"));
        left_button.set_width_chars(10);
        right_button.set_width_chars(10);
        left_button.style_context().add_provider(&buttons_css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        right_button.style_context().add_provider(&buttons_css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        let left_gesture = gtk::GestureClick::new();

        let picture_grid = Grid::new();
        picture_grid.set_widget_name("picture_grid");
        picture_grid.set_row_homogeneous(true);
        picture_grid.set_column_homogeneous(true);
        picture_grid.set_hexpand(true);
        picture_grid.set_vexpand(true);
        if grid_size > 1 {
            panel.attach(&left_button, 0, 0, 1, 1);
            panel.attach(&picture_grid, 1, 0, 1, 1);
            panel.attach(&right_button, 2, 0, 1, 1);
        } else {
            panel.attach(&picture_grid, 0, 0, 1, 1);
        }
        left_gesture.set_button(1);
        left_gesture.connect_pressed(clone!(@strong catalog_rc, @strong picture_grid, @strong picture_grid, @strong application_window => move |_,_,_,_| {
            {
                let mut catalog: RefMut<'_,Catalog> = catalog_rc.borrow_mut();
                catalog.move_prev_page();
            }
            setup_picture_grid(&catalog_rc, &picture_grid, &application_window);
        }));
        left_button.add_controller(left_gesture);
        let right_gesture = gtk::GestureClick::new();
        right_gesture.set_button(1);
        right_gesture.connect_pressed(clone!(@strong catalog_rc, @strong picture_grid, @strong application_window => move |_,_,_,_| {
            {
                let mut catalog: RefMut<'_,Catalog> = catalog_rc.borrow_mut();
                catalog.move_next_page();
            }
            setup_picture_grid(&catalog_rc, &picture_grid, &application_window);
        }));
        right_button.add_controller(right_gesture);
        for col in 0 .. grid_size as i32 {
            for row in 0 .. grid_size as i32 {
                let coords: Coords = (col,row);
                let vbox = gtk::Box::new(Orientation::Vertical, 0);
                vbox.set_valign(Align::Center);
                vbox.set_halign(Align::Center);
                vbox.set_hexpand(true);
                vbox.set_vexpand(true);
                setup_picture_cell(&application_window, &picture_grid, &vbox, coords, &catalog_rc);
                picture_grid.attach(&vbox, col as i32, row as i32, 1, 1);
            }
        }
        grid_scrolled_window.set_child(Some(&panel));

        let gui = Gui {
            application_window: application_window,
            stack: stack,
            grid_scrolled_window: grid_scrolled_window,
            view_scrolled_window: view_scrolled_window,
            picture_grid: picture_grid,
            image_view: image_view,
        };
        gui
}

pub fn setup_picture_cell(window: &gtk::ApplicationWindow, grid: &gtk::Grid, vbox: &gtk::Box, coords: Coords, catalog_rc: &Rc<RefCell<Catalog>>) {
    if let Ok(catalog) = catalog_rc.try_borrow() {
        if let Some(index) = catalog.index_from_position(coords) {
            if let Some(entry) = catalog.entry_at_index(index) {
                if catalog.page_changed() {
                    while let Some(child) = vbox.first_child() {
                        vbox.remove(&child)
                    };
                    let picture = picture_for_entry(entry, &catalog);
                    let label = label_for_entry(entry, index, &catalog);
                    vbox.append(&picture);
                    if catalog.palette_on() { 
                        let drawing_area = drawing_area_for_entry(entry);
                        vbox.append(&drawing_area);
                    }
                    let gesture_left_click = gtk::GestureClick::new();
                    gesture_left_click.set_button(1);
                    gesture_left_click.connect_pressed(clone!(@strong coords, @strong label, @strong entry, @strong catalog_rc, @strong window, @strong grid => move |_,_,_,_| {
                        if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
                            focus_on_cell_at_coords(coords, &grid, &window, &mut catalog, false);
                        }
                    }));
                    picture.add_controller(gesture_left_click);

                    let gesture_right_click = gtk::GestureClick::new();
                    gesture_right_click.set_button(3);
                    gesture_right_click.connect_pressed(clone!(@strong coords, @strong label, @strong catalog_rc, @strong window, @strong grid => move |_,_,_,_| {
                        if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
                            focus_on_cell_at_coords(coords, &grid, &window, &mut catalog, true);
                        }
                    }));
                    picture.add_controller(gesture_right_click);
                    vbox.append(&label);
                }
            }
        } else {
            while let Some(child) = vbox.first_child() {
                vbox.remove(&child)
            }
            let picture = empty_picture();
            let label = empty_label();
            vbox.append(&picture);
            vbox.append(&label);
        }
    } else {
        eprintln!("can't borrow catalog_rc");
    }

}

pub fn empty_picture() -> gtk::Picture {
    gtk::Picture::new()
}

pub fn empty_label() -> gtk::Label {
    let label = gtk::Label::new(None);
    label.set_widget_name("picture_label");
    label
}

pub fn focus_on_cell_at_coords(coords: Coords, grid: &gtk::Grid, window: &gtk::ApplicationWindow, catalog: &mut Catalog, with_select: bool) {
    if catalog.cells_per_row() > 1 {
        if catalog.can_move_abs(coords) {
            set_label_text_at_current_position(&grid, &catalog, false);
            catalog.move_abs(coords);
            if with_select {
                catalog.select_point();
            }
            set_label_text_at_current_position(&grid, &catalog, true);
            window.set_title(Some(&(catalog.title_display())));
        }
    }
}

pub fn set_label_text_at_current_position(grid: &gtk::Grid, catalog: &Catalog, has_focus: bool) {
    let current_coords = catalog.position();
    if let Some(current_entry) = catalog.current_entry() {
        set_label_text_at_coords(grid, current_coords, current_entry.label_display(has_focus, catalog.sample()))
    };
}

pub fn set_label_text_at_coords(grid: &gtk::Grid, coords: Coords, text: String) {
    if let Some(label) = label_at_coords(grid, coords) {
        label.set_text(&text)
    }
}

pub fn label_at_coords(grid: &gtk::Grid, coords: Coords) -> Option<gtk::Label> {
    let (col,row) = coords;
    let vbox = grid.child_at(col as i32, row as i32).expect("can't find a child").downcast::<gtk::Box>().expect("can't downcast child to a Box");
    let child = vbox.first_child().expect("can't access vbox first child").downcast::<gtk::Picture>().expect("can't downcast to Picture");
    let next = child.next_sibling().expect("can't access vbox next child");
    if next.widget_name() == "picture_label" {
        Some(next.downcast::<gtk::Label>().unwrap())
    } else {
        let next_next = next.next_sibling().expect("can't access vbox next next child");
        if next_next.widget_name() == "picture_label" {
            Some(next_next.downcast::<gtk::Label>().unwrap())
        } else {
            panic!("can't find grid picture label");
        }
    }
}

pub fn drawing_area_for_entry(entry: &PictureEntry) -> gtk::DrawingArea {
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_valign(Align::Center);
    drawing_area.set_halign(Align::Center);
    let colors = entry.image_data.palette;
    drawing_area.set_content_width(90);
    drawing_area.set_content_height(10);
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.set_draw_func(move |_, ctx, _, _| {
        draw_palette(ctx, 90, 10, &colors)
    });
    drawing_area
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

pub fn label_for_entry(entry: &PictureEntry, index: usize, catalog: &Catalog) -> gtk::Label {
    let is_current_entry = index == catalog.current_index() && catalog.cells_per_row() > 1;
    let label = gtk::Label::new(Some(&entry.label_display(is_current_entry, catalog.sample())));
    label.set_valign(Align::Center);
    label.set_halign(Align::Center);
    label.set_widget_name("picture_label");
    label
}

pub fn picture_for_entry(entry: &PictureEntry, catalog: &Catalog) -> gtk::Picture {
    let picture = gtk::Picture::new();
    let opacity = if entry.deleted { 0.25 }
    else if entry.selected { 0.50 } else { 1.0 };
    picture.set_valign(Align::Center);
    picture.set_halign(Align::Center);
    picture.set_opacity(opacity);
    picture.set_can_shrink(!catalog.full_size_on());
    let result = if catalog.cells_per_row() < 10 {
        set_original_picture_file(&picture, &entry)
    } else {
        set_thumbnail_picture_file(&picture, &entry)
    };
    match result {
        Ok(_) => picture.set_visible(true),
        Err(err) => {
            picture.set_visible(false);
            eprintln!("{}", err.to_string())
        },
    };
    picture
}

pub fn set_thumbnail_picture_file(picture: &gtk::Picture, entry: &PictureEntry) -> Result<()> {
    let thumbnail = entry.thumbnail_file_path();
    let path = Path::new(&thumbnail);
    if path.exists() {
        picture.set_filename(Some(thumbnail));
        Ok(())
    } else {
        match create_thumbnail(entry) {
            Ok(()) => {
                picture.set_filename(Some(thumbnail));
                Ok(())
            },
            err => err,
        }
    }
}

pub fn set_original_picture_file(picture: &gtk::Picture, entry: &PictureEntry) -> Result<()> {
    let original = entry.original_file_path();
    let path = Path::new(&original);
    if path.exists() {
        picture.set_filename(Some(original));
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Other, format!("file {} doesn't exist", original)))
    }
}

pub fn setup_picture_grid(catalog_rc: &Rc<RefCell<Catalog>>, picture_grid: &gtk::Grid, window: &gtk::ApplicationWindow) {
    if let Ok(catalog) = catalog_rc.try_borrow() {
        let cells_per_row = catalog.cells_per_row();
        for col in 0..cells_per_row as i32 {
            for row in 0..cells_per_row as i32 {
                let vbox = picture_grid.child_at(col,row).unwrap().downcast::<gtk::Box>().unwrap();
                setup_picture_cell(window, &picture_grid, &vbox, (col as usize, row as usize), &catalog_rc);
            }
        }
        window.set_title(Some(&catalog.title_display()));
    }
    else {
        eprintln!("can't borrow catalog_rc");
    }
}

