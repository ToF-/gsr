use clap::builder::PossibleValue;

#[derive(Clone, Debug)]
pub enum Order {
    Colors, Date, Label, Name, Palette, Size, Value, Random,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl clap::ValueEnum for Order {
    fn value_variants<'a>() -> &'a [Self] {
        &[Order::Colors, Order::Date, Order::Name, Order::Random, Order::Size, Order::Value, Order::Palette, Order::Label]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Order::Colors => PossibleValue::new("Colors"),
            Order::Date => PossibleValue::new("Date"),
            Order::Name => PossibleValue::new("Name"),
            Order::Random => PossibleValue::new("Random").help("this is default"),
            Order::Value => PossibleValue::new("Value"),
            Order::Size => PossibleValue::new("Size"),
            Order::Palette => PossibleValue::new("Palette"),
            Order::Label => PossibleValue::new("Label"),
        })
    }
}
