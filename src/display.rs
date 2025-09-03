use std::collections::HashMap;
use crate::rank::Rank;
use crate::catalog::Catalog;
use crate::editor::{Editor, InputKind};
use crate::picture_entry::PictureEntry;
use crate::path::file_path_directory;

pub fn title_display(catalog: &Catalog, editor: &Editor) -> String {
    let entry_display = &<PictureEntry as Clone>::clone(catalog.current_entry().unwrap()).title_display();
    let file_path = &<PictureEntry as Clone>::clone(catalog.current_entry().unwrap()).file_path;
    let display= format!(
        "{}{} S:[{}] {} ordered by {} {}{}/{}{}  {} {} {} {}", 
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
        catalog.selected_count(),
        if catalog.start_index().is_some() { "…" } else { "" },
        if let Some(order) = catalog.order().clone() {
            order.to_string()
        } else {
            "??".to_string()
        },
        if catalog.page_limit_on() { "[" } else { "" },
        catalog.index().unwrap(),
        catalog.last(),
        if catalog.page_limit_on() { "]" } else { "" },
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
pub fn info(catalog: &Catalog) {
    let mut total: f32 = 0.0;
    let mut three_stars: f32 = 0.0;
    let mut two_stars: f32 = 0.0;
    let mut one_stars: f32 = 0.0;
    let mut no_stars: f32 = 0.0;
    let mut labelled: f32 = 0.0;
    let mut labels: HashMap<String,f32> = HashMap::new();
    let mut parents: HashMap<String,usize> = HashMap::new();
    for entry in catalog.picture_entries() {
        total += 1.0;
        match entry.rank {
            Rank::ThreeStars => { three_stars += 1.0 },
            Rank::TwoStars => { two_stars += 1.0 },
            Rank::OneStar => { one_stars += 1.0 }
            Rank::NoStar => { no_stars += 1.0 },
        };
        if entry.label().is_some() {
            labelled += 1.0;
            if let Some(number) = labels.get_mut(&entry.label().unwrap()) {
                *number = *number + 1.0
            } else {
                labels.insert(entry.label().unwrap().clone(), 1.0);
            }
        };
        let parent:String = entry.parent_path();
        if let Some(count) = parents.get_mut(&parent) {
            *count = *count + 1
        } else {
            parents.insert(parent, 1);
        }
    }
    println!("total: {}", total);
    println!("{}: {} ({:.2}%)", Rank::ThreeStars.to_string(), three_stars, three_stars / total * 100.0);
    println!("{}: {} ({:.2}%)", Rank::TwoStars.to_string(), two_stars, two_stars / total * 100.0);
    println!("{}: {} ({:.2}%)", Rank::OneStar.to_string(), one_stars, one_stars / total * 100.0);
    println!("{}: {} ({:.2}%)", Rank::NoStar.to_string(), no_stars, no_stars / total * 100.0);
    println!("labelled: {} ({:.2}%)", labelled, labelled / total * 100.0);
    let mut all_labels = Vec::from_iter(labels.keys());
    all_labels.sort();
    for key in all_labels.iter() {
        if let Some(val) = labels.get(&key as &str) {
            println!("{key}:{val}")
        } else {
        };
    }
    let mut all_parents :Vec<(usize,String)> = Vec::new();
    for(key,val) in parents.iter() {
        all_parents.push((*val,key.to_string()));
    }
    all_parents.sort_by(|a,b| a.0.cmp(&b.0));
    for (val,key) in all_parents.iter() {
        println!("{val:>12} {key}");
    }
}

