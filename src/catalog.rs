
pub struct Catalog {
}

impl Catalog {

    pub fn new() -> Self {
        Catalog { }
    }

    pub fn length(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_creation_length_is_0() {
        let catalog = Catalog::new();
        assert_eq!(catalog.length(), 0);
    }
}
