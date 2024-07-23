use crate::Args;
use std::cell::{RefCell};
use std::rc::Rc;

pub fn build_gui(args: &Args, application: &gtk::Application) {
    let width:  i32 = args.width.unwrap();
    let height: i32 = args.height.unwrap();
    let grid_size = args.grid.unwrap();
    match init_catalog(args) {
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


