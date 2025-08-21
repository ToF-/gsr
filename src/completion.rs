use compare::{Compare, natural};
use std::cmp::Ordering::{Less, Equal, Greater};

pub fn candidates(prefix: &String, labels: &Vec<String>) -> Vec<String> {
    let mut bottom: usize;
    let mut top: usize;
    let mut middle: usize;
    let mut start: usize = 0;
    let mut end: usize = 0;
    bottom = 0;
    top = labels.len();
    while bottom <= top {
        let cmp = natural();
        middle = (bottom + top) / 2;
        println!("{} {} {} {}", bottom, top, middle, labels[middle]);
        match cmp.compare(prefix, &labels[middle]) {
            Greater => {
                bottom = middle + 1;
            },
            Equal => {
                start = middle;
                while labels[middle].starts_with(prefix) && middle < labels.len() {
                    middle += 1;
                }
                end = middle - 1;
                return labels[start..end].to_vec()
            },
            Less => {
                top = middle - 1;
            }
        }
        println!("{} {} {} {}", bottom, top, middle, labels[middle]);
        if middle < labels.len() && labels[middle].starts_with(prefix) {
            while labels[middle].starts_with(prefix) && middle < labels.len() {
                middle += 1;
            }
            end = middle - 1;
            return labels[start..end].to_vec()
        }
        else {
            return vec![];
        }
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_one_candidate() {
        let mut labels: Vec<String> = vec!["car".into(), "caring".into(), "capital".into(), "barge".into()];
        labels.sort();
        assert_eq!(vec![String::from("capital")], candidates(&"capi".into(), &labels));
    }
}
