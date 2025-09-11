use crate::display::title_display;
use crate::editor::{Editor,InputKind};
use crate::glib::timeout_add_local;
use anyhow::{Result};
use crate::commands::{Command,Shortcuts, export_shortcuts};
use std::time::Duration;
use gtk::{Align, ApplicationWindow, CssProvider, Grid, gdk, Label, Orientation, Picture, ScrolledWindow};
use crate::rank::Rank;
use gtk::cairo::{Context, Format, ImageSurface};
use gtk::gdk::Key;
use crate::direction::Direction;
use crate::Args;
use crate::order;
use crate::Catalog;
use crate::picture_entry::PictureEntry;
use crate::picture_io::check_or_create_thumbnail_file;
use gtk::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::glib::clone;

struct Gui {
    application_window: gtk::ApplicationWindow,
    single_view_scrolled_window: gtk::ScrolledWindow,
    multiple_view_scrolled_window: gtk::ScrolledWindow,
    multiple_view_grid: gtk::Grid,
    view_stack: gtk::Stack,
    single_view_box: gtk::Box,
    single_view_picture: gtk::Picture,
    cells_per_row: i32,
    shortcuts: Shortcuts,
    editor: Editor,
}

impl Gui {
    pub fn single_view_mode(&self) -> bool {
        let child = self.view_stack.visible_child().expect("view stack has no child");
        child == self.single_view_scrolled_window
    }

    pub fn cell_box_at(&self, col: i32, row: i32) -> gtk::Box {
        let widget = self.multiple_view_grid.child_at(col, row)
            .unwrap_or_else(||
                panic!("cannot find child at {} {}", col, row));
        widget.downcast::<gtk::Box>().expect("cannot downcast widget to Box")
    }
}

pub fn build_gui(application: &gtk::Application, args: &Args, catalog_rc: &Rc<RefCell<Catalog>>, shortcuts: &Shortcuts) {
    let cells_per_row: i32 = match catalog_rc.try_borrow() {
        Ok(catalog) => catalog.navigator().cells_per_row() as i32,
        Err(err) => panic!("{}", err),
    };
    let width = args.width.unwrap();
    let height = args.height.unwrap();
    let application_window = ApplicationWindow::builder()
        .application(application)
        .title("gsr")
        .default_width(width)
        .default_height(height)
        .build();

    let single_view_scrolled_window = ScrolledWindow::builder()
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
    single_view_scrolled_window.set_child(Some(&view_box));

    let multiple_view_scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .name("grid")
        .build();

    let multiple_view_panel =Grid::new();
    multiple_view_panel.set_hexpand(true);
    multiple_view_panel.set_vexpand(true);
    multiple_view_panel.set_row_homogeneous(true);
    multiple_view_panel.set_column_homogeneous(false);
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
    let left_button = Label::new(Some("←"));
    let right_button= Label::new(Some("→"));
    left_button.set_width_chars(10);
    right_button.set_width_chars(10);
    left_button.style_context().add_provider(&buttons_css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    right_button.style_context().add_provider(&buttons_css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let multiple_view_grid = Grid::new();
    multiple_view_grid.set_widget_name("multiple_view_grid");
    multiple_view_grid.set_row_homogeneous(true);
    multiple_view_grid.set_column_homogeneous(true);
    multiple_view_grid.set_hexpand(true);
    multiple_view_grid.set_vexpand(true);
    multiple_view_panel.attach(&left_button, 0, 0, 1, 1);
    multiple_view_panel.attach(&multiple_view_grid, 1, 0, 1, 1);
    multiple_view_panel.attach(&right_button, 2, 0, 1, 1);

    if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
        catalog.refresh()
    };
    for col in 0 .. cells_per_row {
        for row in 0 .. cells_per_row {
            let cell_box = gtk::Box::new(Orientation::Vertical, 0);
            cell_box.set_valign(Align::Center);
            cell_box.set_halign(Align::Center);
            cell_box.set_hexpand(true);
            cell_box.set_vexpand(true);
            setup_picture_cell(&cell_box, col, row, catalog_rc);
            multiple_view_grid.attach(&cell_box, col, row, 1, 1);
            assert!(multiple_view_grid.child_at(col, row).unwrap() == cell_box);
        }
    }
    multiple_view_scrolled_window.set_child(Some(&multiple_view_panel));

    let view_stack = gtk::Stack::new();
    view_stack.set_hexpand(true);
    view_stack.set_vexpand(true);
    let _ = view_stack.add_child(&single_view_scrolled_window);
    let _ = view_stack.add_child(&multiple_view_scrolled_window);
    if cells_per_row > 1 {
        view_stack.set_visible_child(&multiple_view_scrolled_window);
    } else {
        view_stack.set_visible_child(&single_view_scrolled_window);
    }
    application_window.set_child(Some(&view_stack));

    let gui = Gui {
        application_window,
        single_view_scrolled_window,
        multiple_view_scrolled_window,
        multiple_view_grid,
        view_stack,
        single_view_box: view_box,
        single_view_picture: picture,
        cells_per_row,
        shortcuts: shortcuts.clone(),
        editor: Editor::new(),
    };
    let gui_rc = Rc::new(RefCell::new(gui));

    if let Ok(gui) = gui_rc.try_borrow() {
        for col in 0 .. gui.cells_per_row {
            for row in 0 .. gui.cells_per_row {
                let cell_box = gui.cell_box_at(col, row);
                let gesture_left_click = gtk::GestureClick::new();
                gesture_left_click.set_button(1);
                gesture_left_click.connect_pressed(clone!(@strong col, @strong row, @strong gui_rc, @strong catalog_rc => move |_,_,_,_| {
                    if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
                        if let Ok(gui) = gui_rc.try_borrow() {
                            let _ = left_click_command_view_mode(col as usize, row as usize, &gui, &mut catalog);
                        }
                    }
                }));
                cell_box.add_controller(gesture_left_click);
                let gesture_right_click = gtk::GestureClick::new();
                gesture_right_click.set_button(3);
                gesture_right_click.connect_pressed(clone!(@strong col, @strong row, @strong gui_rc, @strong catalog_rc => move |_,_,_,_| {
                    if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
                        if let Ok(gui) = gui_rc.try_borrow() {
                            let _ = right_click_command_view_mode(col as usize, row as usize, &gui, &mut catalog);
                        }
                    }
                }));
                cell_box.add_controller(gesture_right_click);
            }
        }
    }
    let left_gesture = gtk::GestureClick::new();
    left_gesture.set_button(1);
    left_gesture.connect_pressed(clone!(@strong catalog_rc, @strong gui_rc => move |_,_,_,_| {
        {
            let mut catalog = catalog_rc.borrow_mut();
            catalog.mut_navigator().move_prev_page();
        }
        if let Ok(catalog) =  catalog_rc.try_borrow() {
            if let Ok(gui) = gui_rc.try_borrow() {
                refresh_view(&gui, &catalog);
            }
        }
    }));
    left_button.add_controller(left_gesture);

    let right_gesture = gtk::GestureClick::new();
    right_gesture.set_button(1);
    right_gesture.connect_pressed(clone!(@strong catalog_rc, @strong gui_rc => move |_,_,_,_| {
        {
            let mut catalog = catalog_rc.borrow_mut();
            catalog.mut_navigator().move_next_page();
        }
        if let Ok(catalog) =  catalog_rc.try_borrow() {
            if let Ok(gui) = gui_rc.try_borrow() {
                refresh_view(&gui, &catalog);
            }
        }
    }));
    right_button.add_controller(right_gesture);

    let evk = gtk::EventControllerKey::new();
    evk.connect_key_pressed(clone!(@strong catalog_rc, @strong gui_rc => move |_, key, _, _| {
        process_key(&catalog_rc, &gui_rc, key) 
    }));
    if let Some(seconds) = args.seconds {
        timeout_add_local(Duration::new(seconds, 0), clone!(@strong catalog_rc, @strong gui_rc => move | | {
            if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
                if let Ok(gui) = gui_rc.try_borrow() {
                    catalog.mut_navigator().move_next_page();
                    refresh_view(&gui, &catalog);
                }
            };
            Continue(true)
        }));
    };
    if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
        if let Ok(gui) = gui_rc.try_borrow() {
            gui.application_window.add_controller(evk);
            catalog.mut_navigator().move_to_previous_index();
            catalog.refresh();
            refresh_view(&gui, &catalog);
            gui.application_window.present()
        }
    };
}

pub fn startup_gui(_application: &gtk::Application) {
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data("window { background-color:black;} image { margin:1em ; } label { color:white; }");
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &css_provider,
        1000,
    );
}

fn process_key(catalog_rc: &Rc<RefCell<Catalog>>, gui_rc: &Rc<RefCell<Gui>>, key: Key) -> gtk::Inhibit {
    if let Ok(mut catalog) = catalog_rc.try_borrow_mut() {
        if let Ok(mut gui) = gui_rc.try_borrow_mut() {
            let refresh: bool = if gui.editor.editing() {
                let refresh = input_mode_process_key(key, &mut gui.editor, &mut catalog);
                set_title(&gui, &catalog);
                refresh
            } else if catalog.sort_selection_on() {
                sort_selection_process_key(key, &mut catalog)
            } else if catalog.args().unwrap().covers {
                sample_mode_process_key(key, &mut gui, &mut catalog)
            } else {
                view_mode_process_key(key, &mut gui, &mut catalog)
            };
            if refresh { refresh_view(&gui, &catalog) }
        }
    }
    gtk::Inhibit(false)
}

fn refresh_view(gui: &Gui, catalog: &Catalog) {
    if let Some(child) = gui.view_stack.visible_child() {
        if child == gui.single_view_scrolled_window {
            set_picture_for_single_view(gui, catalog)
        } else {
            set_all_pictures_for_multiple_view(gui, catalog)
        }
    };
    set_title(gui, catalog);
}

fn input_mode_process_key(key: Key, editor: &mut Editor, catalog: &mut Catalog) -> bool {
    let mut refresh: bool = false;
    match key.name() {
        None => refresh = false,
        Some(key_name) => match key_name.as_str() {
            "Escape" => {
                editor.cancel();
            },
            "Return" => {
                editor.confirm(catalog);
                refresh = true
            },
            "BackSpace" => {
                editor.delete();
            },
            "Tab" => {
                editor.complete();
            },
            _ => {
                if let Some(ch) = key.to_unicode() {
                    editor.append(ch);
                }
            }
        },
    };
    refresh
}

fn sort_selection_process_key(key: Key, catalog: &mut Catalog) -> bool {
    let mut refresh: bool = true;
    match key.name() {
        None => refresh = false,
        Some(key_name) => match key_name.as_str() {
            "Escape" => catalog.cancel_sort_selection(),
            s => if let Some(order) = order::from(s) { catalog.sort_by(order) },
        },
    };
    refresh
}

fn sample_mode_process_key(key: Key, gui: &mut Gui, catalog: &mut Catalog) -> bool {
    view_mode_process_key(key, gui, catalog)
}

fn view_mode_process_key(key: Key, gui: &mut Gui, catalog: &mut Catalog) -> bool {
    let mut refresh: bool = true;
    match key.name() {
        None => refresh = false,
        Some(key_name) =>
            match gui.shortcuts.get(&key_name.to_string()) {
                None => println!("{}", key_name),
                Some(command) => {
                    let mut result: Result<()> = Ok(());
                    match command {
                        Command::NoStar => {
                            let _ = catalog.rank_current_entry(Rank::NoStar);
                        },
                        Command::OneStar => {
                            let _ = catalog.rank_current_entry(Rank::OneStar);
                        },
                        Command::TwoStars => {
                            let _ = catalog.rank_current_entry(Rank::TwoStars);
                        },
                        Command::ThreeStars => {
                            let _ = catalog.rank_current_entry(Rank::ThreeStars);
                        },
                        Command::Cover => result = catalog.cover_current_entry(),
                        Command::Extract => catalog.extract(),
                        Command::FirstPosition => refresh = left_click_command_view_mode(0,0, gui, catalog),
                        Command::LastPosition => refresh = left_click_command_view_mode(catalog.navigator().cells_per_row()-1, catalog.navigator().cells_per_row()-1, gui, catalog),
                        Command::CopyLabel => catalog.copy_label(),
                        Command::CopyTemp => result = catalog.copy_picture_file_to_temp(),
                        Command::Delete => {
                            gui.editor.delete();
                            result = catalog.toggle_delete_current_entry()
                        },
                        Command::ToggleExpand => catalog.toggle_expand(),
                        Command::ToggleFullSize => if gui.single_view_mode() {
                            catalog.toggle_full_size()
                        },
                        Command::GotoIndex => {
                            gui.editor.begin_input(InputKind::Index, catalog.tags.clone());
                        },
                        Command::GridTwo => {
                            catalog.set_new_page_size(2);
                            gui.application_window.close()
                        },
                        Command::GridThree => {
                            catalog.set_new_page_size(3);
                            gui.application_window.close()
                        },
                        Command::GridFour => {
                            catalog.set_new_page_size(4);
                            gui.application_window.close()
                        },
                        Command::GridFive => {
                            catalog.set_new_page_size(5);
                            gui.application_window.close()
                        },
                        Command::GridTen => {
                            catalog.set_new_page_size(10);
                            gui.application_window.close()
                        },
                        Command::Random => catalog.mut_navigator().move_to_random_index(),
                        Command::Info => catalog.print_info(&gui.editor),
                        Command::Jump => { 
                            gui.editor.begin_input(InputKind::SearchLabel, catalog.tags.clone());
                        },
                        Command::ExportCommands => result = export_shortcuts(&gui.shortcuts),
                        Command::NextPage => catalog.mut_navigator().move_next_page(),
                        Command::TogglePageLimit => catalog.mut_navigator().toggle_page_limit(),
                        Command::PrevPage => catalog.mut_navigator().move_prev_page(),
                        Command::QuitWithCancel => {
                            catalog.exit();
                            gui.application_window.close()
                        },
                        Command::QuitWithConfirm => {
                            let _ = catalog.redirect_files();
                            let _ = catalog.delete_files();
                            catalog.exit();
                            gui.application_window.close()
                        },
                        Command::Repeat => result = catalog.end_repeat_last_comment(),
                        Command::Search => {
                            gui.editor.begin_input(InputKind::Search, catalog.tags.clone());
                        }
                        Command::Uncover => result = catalog.uncover_current_entry(),
                        Command::UnSelectPage => result = catalog.unselect_page(),
                        Command::UnselectAll => result = catalog.unselect_all(),
                        Command::TogglePalette => {
                            catalog.toggle_palette();
                            set_title(gui, catalog);
                        },
                        Command::StartPosition => catalog.mut_navigator().move_to_first_index(),
                        Command::EndPosition => catalog.mut_navigator().move_to_last_index(),
                        Command::Next => catalog.mut_navigator().move_next_page(),
                        Command::SetRange => catalog.start_set(),
                        Command::Cancel => catalog.cancel_set(),
                        Command::PasteLabel => result = catalog.paste_label_current_entry(),
                        Command::Unlabel => result = catalog.unlabel_current_entry(),
                        Command::LabelTag => result = catalog.label_tag_current_entry(),
                        Command::ToggleSingleView => if catalog.page_size() > 1 {
                            if gui.single_view_mode() {
                                gui.view_stack.set_visible_child(&gui.multiple_view_scrolled_window);
                            } else {
                                gui.view_stack.set_visible_child(&gui.single_view_scrolled_window);
                            }
                        },
                        Command::ChooseOrder => catalog.begin_sort_selection(),
                        Command::ToggleSelect => {
                            result = catalog.toggle_select_current_entry();
                            catalog.count_selected()
                        },
                        Command::AddTag => {
                            gui.editor.begin_input(InputKind::AddTag, catalog.tags.clone());
                        }
                        Command::DeleteTag => {
                            gui.editor.begin_input(InputKind::DeleteTag, catalog.current_entry().unwrap().tags.clone());
                        }
                        Command::Label => {
                            gui.editor.begin_input(InputKind::Label, catalog.tags.clone());
                        }
                        Command::Relabel => {
                            gui.editor.begin_input(InputKind::Relabel, catalog.tags.clone());
                        }
                        Command::Right => {
                            refresh = arrow_command(Direction::Right, gui, catalog)
                        },
                        Command::Left => {
                            refresh = arrow_command(Direction::Left, gui, catalog)
                        },
                        Command::Down => {
                            refresh = arrow_command(Direction::Down, gui, catalog)
                        },
                        Command::Up => {
                            refresh = arrow_command(Direction::Up, gui, catalog)
                        },
                    };
                    if result.is_err() {
                        eprintln!("{}", result.unwrap_err())
                    }
                },
            },
    };
    refresh
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

fn draw_palette(ctx: &Context, width: i32, height: i32, colors: &[u32;9]) {
    const COLOR_MAX: f64 = 9.0;
    let square_size: f64 = height as f64;
    let offset: f64 = (width as f64 - (COLOR_MAX * square_size)) / 2.0;
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
fn set_title(gui: &Gui, catalog: &Catalog) {
    gui.application_window.set_title(Some(&title_display(catalog, &gui.editor)))
}

fn arrow_command_full_size(direction: Direction, gui: &Gui) -> bool {
    let step: f64 = 100.0;
    let (picture_adjustment, step) = match direction {
        Direction::Right => (gui.single_view_scrolled_window.hadjustment(), step),
        Direction::Left  => (gui.single_view_scrolled_window.hadjustment(), -step),
        Direction::Down  => (gui.single_view_scrolled_window.vadjustment(), step),
        Direction::Up    => (gui.single_view_scrolled_window.vadjustment(), -step),
    };
    picture_adjustment.set_value(picture_adjustment.value() + step);
    false
}

fn set_picture_for_single_view(gui: &Gui, catalog: &Catalog) {
    let view_box = &gui.single_view_box;
    let picture = &gui.single_view_picture;
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
    if let Some(widget) = view_box.last_child() {
        if widget != *picture {
            view_box.remove(&widget)
        }
    }
    if catalog.palette_on() {
        let colors = entry.palette;
        let palette_area = create_palette(colors);
        view_box.insert_child_after(&palette_area, Some(picture));
    }
}

fn set_picture_for_cell_at(gui: &Gui, catalog: &Catalog, col: usize, row: usize) {
    let widget = gui.multiple_view_grid.child_at(col as i32, row as i32).expect("cannot find cell box in multiple view grid");
    let cell_box = widget.downcast::<gtk::Box>().expect("cannot downcast widget to Box");

    while let Some(child) = cell_box.first_child() {
        cell_box.remove(&child)
    };
    if let Some(index) = catalog.index_from_position((col,row)) {
        if !catalog.discarded().contains(&index) {
            let entry = catalog.entry_at_index(index).unwrap();
            let picture = picture_for_entry(entry, catalog);
            let label = label_for_entry(entry, index == catalog.index().unwrap());
            cell_box.append(&picture);
            cell_box.append(&label);
        }
    }
}

fn set_label_for_cell_index(gui: &Gui, catalog: &Catalog, index: usize, has_focus: bool) {
    let (col,row) = catalog.position_from_index(index);
    let widget = gui.multiple_view_grid.child_at(col as i32, row as i32).expect("cannot find cell box in multiple view grid");
    let cell_box = widget.downcast::<gtk::Box>().expect("cannot downcast widget to Box");
    if let Some(child) = cell_box.last_child() {
        cell_box.remove(&child)
    };
    let entry = catalog.entry_at_index(index).unwrap();
    let label = label_for_entry(entry, has_focus);
    cell_box.append(&label);
}

fn set_all_pictures_for_multiple_view(gui: &Gui, catalog: &Catalog) {
    for col in 0..catalog.page_size() {
        for row in 0..catalog.page_size() {
            set_picture_for_cell_at(gui, catalog, col, row)
        }
    }
}

fn arrow_command_view_mode(direction: Direction, gui: &Gui, catalog: &mut Catalog) -> bool {
    let old_index: usize = catalog.index().unwrap();
    let old_page_index: usize = catalog.navigator().page_index();
    if catalog.navigator().can_move_towards(direction.clone()) {
        catalog.mut_navigator().move_towards(direction);
        set_picture_for_single_view(gui, catalog);
        let new_index = catalog.index().unwrap();
        if catalog.navigator().page_index() != old_page_index {
            set_all_pictures_for_multiple_view(gui, catalog)
        } else {
            set_label_for_cell_index(gui, catalog, old_index, false)
        };
        set_label_for_cell_index(gui, catalog, new_index, true);
        false
    } else {
        false
    }
}

fn left_click_command_view_mode(col: usize, row: usize, gui: &Gui, catalog: &mut Catalog) -> bool {
    let old_index: usize = catalog.index().unwrap();
    let old_page_index: usize = catalog.navigator().page_index();
    if let Some(new_index) = catalog.index_from_position((col, row)) {
        if catalog.navigator().can_move_to_index(new_index) {
            catalog.mut_navigator().move_to_index(new_index);
            set_picture_for_single_view(gui, catalog);
            if catalog.navigator().page_index() != old_page_index {
                set_all_pictures_for_multiple_view(gui, catalog)
            } else {
                set_label_for_cell_index(gui, catalog, old_index, false)
            };
            set_label_for_cell_index(gui, catalog, new_index, true);
            false
        } else {
            false
        }
    } else {
        false
    }
}

fn right_click_command_view_mode(col: usize, row: usize, gui: &Gui, catalog: &mut Catalog) -> bool {
    let old_index: usize = catalog.index().unwrap();
    let old_page_index: usize = catalog.navigator().page_index();
    if let Some(new_index) = catalog.index_from_position((col, row)) {
        catalog.start_set();
        if catalog.navigator().can_move_to_index(new_index) {
            catalog.mut_navigator().move_to_index(new_index);
            let _ = catalog.toggle_select_current_entry();
            set_picture_for_single_view(gui, catalog);
            if catalog.navigator().page_index() != old_page_index {
                set_all_pictures_for_multiple_view(gui, catalog)
            } else {
                set_label_for_cell_index(gui, catalog, old_index, false)
            };
            set_label_for_cell_index(gui, catalog, new_index, true);
            false
        } else {
            false
        }
    } else {
        false
    }
}

fn arrow_command(direction: Direction, gui: &Gui, catalog: &mut Catalog) -> bool {
    if gui.single_view_mode() && catalog.full_size_on() {
        arrow_command_full_size(direction, gui)
    } else {
        let _ = arrow_command_view_mode(direction, gui, catalog);
        gui.application_window.set_title(Some(&title_display(catalog, &gui.editor)));
        false
    }
}

fn setup_picture_cell(cell_box: &gtk::Box, col: i32, row: i32, catalog_rc: &Rc<RefCell<Catalog>>) {
    if let Ok(catalog) = catalog_rc.try_borrow() {
        let coords = (col as usize, row as usize);
        if let Some(index) = catalog.index_from_position(coords) {
            if catalog.page_changed() {
                while let Some(child) = cell_box.first_child() {
                    cell_box.remove(&child)
                };
                let entry = catalog.entry_at_index(index).unwrap();
                let picture = picture_for_entry(entry, &catalog);
                let label = label_for_entry(entry, index == catalog.index().unwrap());
                cell_box.append(&picture);
                cell_box.append(&label);
            }
        }
    }
}

fn picture_for_entry(entry: &PictureEntry, catalog: &Catalog) -> gtk::Picture {
    let picture = gtk::Picture::new();
    let opacity = if entry.deleted { 0.25 }
    else if entry.selected { 0.50 } else { 1.0 };
    picture.set_valign(Align::Center);
    picture.set_halign(Align::Center);
    picture.set_opacity(opacity);
    picture.set_can_shrink(!catalog.full_size_on());
    if catalog.navigator().cells_per_row() < 10 {
        picture.set_filename(Some(entry.original_file_path()));
    } else {
        let _ = check_or_create_thumbnail_file(&entry.thumbnail_file_path(), &entry.original_file_path());
        picture.set_filename(Some(entry.thumbnail_file_path()));
    };
    picture.set_visible(true);
    picture
}
fn label_for_entry(entry: &PictureEntry, with_focus: bool) -> gtk::Label {
    let label = gtk::Label::new(Some(&entry.label_display(with_focus)));
    label.set_valign(Align::Center);
    label.set_halign(Align::Center);
    label.set_widget_name("picture_label");
    label
}
