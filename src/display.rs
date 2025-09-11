use std::collections::HashMap;
use crate::rank::Rank;
use crate::catalog::Catalog;
use crate::editor::{Editor, InputKind};
use crate::picture_entry::PictureEntry;
use crate::path::file_path_directory;

pub fn directory_display(catalog: &Catalog, file_path: &str) -> String {
    if catalog.args().unwrap().covers {
        file_path_directory(file_path).to_string()
    } else {
        String::from("")
    }
}

pub fn editor_input_display(editor: &Editor) -> String {
    match editor.input_kind() {
            Some(InputKind::AddTag) => format!("add tag:{} {}", editor.input(), editor.candidates()),
            Some(InputKind::DeleteTag) => format!("delete tag:{} {}", editor.input(), editor.candidates()),
            Some(InputKind::Search) => format!("search:{}", editor.input()),
            Some(InputKind::SearchLabel) => format!("label search:{} {}", editor.input(), editor.candidates()),
            Some(InputKind::Label) => format!("label:{} {}", editor.input(), editor.candidates()),
            Some(InputKind::Relabel) => format!("relabel:{} {}", editor.input(), editor.candidates()),
            Some(InputKind::Index) => format!("index:{}", editor.input()),
            None => String::from(""),
    }
}

pub fn title_display(catalog: &Catalog, editor: &Editor) -> String {
    let entry_display = &<PictureEntry as Clone>::clone(catalog.current_entry().unwrap()).title_display();
    let file_path = &<PictureEntry as Clone>::clone(catalog.current_entry().unwrap()).file_path;
    let display= format!(
        "{} S:{} {} ordered by {} {}{}/{}{}  {} {} {} {}", 
        directory_display(catalog, file_path),
        catalog.selected_count(),
        if catalog.navigator().start_index().is_some() { "…" } else { "" },
        if let Some(order) = catalog.order() { order.to_string() } else { "??".to_string() },
        if catalog.navigator().page_limit_on() { "[" } else { "" },
        catalog.index().unwrap(),
        catalog.last(),
        if catalog.navigator().page_limit_on() { "]" } else { "" },
        entry_display,
        if catalog.expand_on() { "□" } else { "" },
        if catalog.full_size_on() { "░" } else { "" },
        editor_input_display(editor)
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
                *number += 1.0
            } else {
                labels.insert(entry.label().unwrap().clone(), 1.0);
            }
        };
        let parent:String = entry.parent_path();
        if let Some(count) = parents.get_mut(&parent) {
            *count += 1
        } else {
            parents.insert(parent, 1);
        }
    }
    println!("total: {}", total);
    println!("{}: {} ({:.2}%)", Rank::ThreeStars, three_stars, three_stars / total * 100.0);
    println!("{}: {} ({:.2}%)", Rank::TwoStars, two_stars, two_stars / total * 100.0);
    println!("{}: {} ({:.2}%)", Rank::OneStar, one_stars, one_stars / total * 100.0);
    println!("{}: {} ({:.2}%)", Rank::NoStar, no_stars, no_stars / total * 100.0);
    println!("labelled: {} ({:.2}%)", labelled, labelled / total * 100.0);
    let mut all_labels = Vec::from_iter(labels.keys());
    all_labels.sort();
    for key in all_labels.iter() {
        if let Some(val) = labels.get(key as &str) {
            println!("{key}:{val}")
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

