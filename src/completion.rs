use std::collections::HashSet;

pub fn candidates(prefix: &String, labels: &HashSet<String>) -> Vec<String> {
    let mut result: Vec<String> = labels
        .iter()
        .filter(|label| label.starts_with(prefix))
        .cloned()
        .collect::<Vec<String>>();
    result.sort();
    result

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_one_candidate() {
        let labels: HashSet<String> = HashSet::from(["facile".into(), "facio".into(), "factum".into(),"fidens".into(),"fideis".into(),"forte".into()]);
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
