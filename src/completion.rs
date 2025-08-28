use std::collections::HashSet;

pub fn candidates(prefix: &String, labels: &HashSet<String>) -> Vec<String> {
    labels.into_iter().filter(|label| label.starts_with(prefix)).map(|s| s.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_one_candidate() {
        let mut labels: Vec<String> = vec!["facile".into(), "facio".into(), "factum".into(),"fidens".into(),"fideis".into(),"forte".into()];
        labels.sort();
        let empty: Vec<String> = vec![];
        assert_eq!(empty, candidates(&"enea".into(), &labels));
        let expected_f:Vec<String> = vec!["facile".into(), "facio".into(), "factum".into(), "fideis".into(), "fidens".into(), "forte".into()];
        assert_eq!(expected_f, candidates(&"f".into(), &labels));
        let expected_fa:Vec<String> = vec!["facile".into(), "facio".into(), "factum".into()];
        assert_eq!(expected_fa, candidates(&"fa".into(), &labels));
        let expected_fact:Vec<String> = vec!["factum".into()];
        assert_eq!(expected_fact, candidates(&"fact".into(), &labels));
    }
}
