use crate::catalog::Catalog;
use crate::editor::{Editor, InputKind};
use crate::picture_entry::PictureEntry;
use crate::path::file_path_directory;

pub fn title_display(catalog: &Catalog, editor: &Editor) -> String {
    let entry_display = &<PictureEntry as Clone>::clone(catalog.current_entry().unwrap()).title_display();
    let file_path = &<PictureEntry as Clone>::clone(catalog.current_entry().unwrap()).file_path;
    let display= format!("{}{} S:[{}] {} ordered by {} {}/{}  {} {} {} {}",
        if catalog.db_centric() {
            String::from("◯")
        } else {
            String::from("▻")
        },
        if catalog.args().unwrap().covers {
            file_path_directory(file_path)
        } else {
            String::from("")
        },
        catalog.max_selected(),
        if catalog.start_index().is_some() { "…" } else { "" },
        if let Some(order) = catalog.order().clone() {
            order.to_string()
        } else {
            "??".to_string()
        },
        catalog.index().unwrap(),
        catalog.last(),
        entry_display,
        if catalog.expand_on() { "□" } else { "" },
        if catalog.full_size_on() { "░" } else { "" },
        if let Some(kind) = editor.input_kind() {
            match kind {
                InputKind::AddTagInput => format!("add tag:{} {}", editor.input(), catalog.current_candidates()),
                InputKind::DeleteTagInput => format!("delete tag:{} {}", editor.input(), catalog.current_candidates()),
                InputKind::SearchInput => format!("search:{}", editor.input()),
                InputKind::SearchLabelInput => format!("label search:{} {}", editor.input(), catalog.current_candidates()),
                InputKind::LabelInput => format!("label:{} {}", editor.input(), catalog.current_candidates()),
                InputKind::RelabelInput => format!("relabel:{} {}", editor.input(), catalog.current_candidates()),
                InputKind::IndexInput => format!("index:{}", editor.input()),
            }
        } else {
            String::from("")
        }
    );
    display
}
