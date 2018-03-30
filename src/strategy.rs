use rand::{self, Rng};

/// The identifier allocation strategy to use at a specified depth.
#[derive(Clone, PartialEq)]
pub enum LSEQStrategy {
  /// Generate identifiers by adding a value to the previous digit.
  AddFromLeft,
  /// Generate identifiers by subtracting a value to the next digit.
  SubtractFromRight,
}

impl LSEQStrategy {
  pub fn random() -> LSEQStrategy {
    let mut rng = rand::thread_rng();
    rng
      .choose(&[LSEQStrategy::AddFromLeft, LSEQStrategy::SubtractFromRight])
      .unwrap()
      .clone()
  }
}
