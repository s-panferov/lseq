use rug::Integer;
use rug::ops::Pow;

pub trait BitBase {
  fn get_bits(&self, depth: u32) -> u32;
  fn get_base(&self, depth: u32) -> u32;
  fn get_max(&self, depth: u32) -> Integer;
  fn get_initial_base(&self) -> u32;
  fn interval(&self, left: &Integer, right: &Integer, depth: u32) -> Integer;
}

pub struct DoubleBase {
  initial: u32,
}

impl DoubleBase {
  pub fn new<B: Into<Option<u32>>>(initial: B) -> DoubleBase {
    DoubleBase {
      initial: initial.into().unwrap_or(5),
    }
  }
}

impl BitBase for DoubleBase {
  fn get_initial_base(&self) -> u32 {
    self.initial
  }

  fn get_base(&self, depth: u32) -> u32 {
    self.initial + depth
  }

  fn get_max(&self, depth: u32) -> Integer {
    let base = self.get_base(depth);
    Integer::from(2).pow(base) - 1
  }

  fn get_bits(&self, depth: u32) -> u32 {
    let n = self.get_base(depth);
    let m = self.initial - 1;
    (n * (n + 1)) / 2 - (m * (m + 1) / 2)
  }

  fn interval(&self, left: &Integer, right: &Integer, depth: u32) -> Integer {
    let left_len = left.significant_bits() - 1;
    let right_len = right.significant_bits() - 1;
    let total = self.get_bits(depth);

    let left_norm: Integer = if left_len < total {
      Integer::from(left << (total - left_len))
    } else {
      Integer::from(left >> (left_len - total))
    };

    let right_norm: Integer = if right_len < total {
      Integer::from(right << (total - right_len))
    } else {
      Integer::from(right >> (right_len - total))
    };

    right_norm - left_norm - 1
  }
}

#[cfg(test)]
mod tests {
  use super::{BitBase, DoubleBase};
  use rug::Integer;

  #[test]
  fn it_works() {
    let base = DoubleBase::new(None);
    assert_eq!(base.get_initial_base(), 5);
    assert_eq!(base.get_base(0), 5);
    assert_eq!(base.get_max(0), 31);
    assert_eq!(base.get_max(1), 63);
    assert_eq!(base.get_max(2), 127);
    assert_eq!(base.get_bits(0), 5);
    assert_eq!(base.get_bits(1), 11);

    let mut left = Integer::from(0);
    let mut right = Integer::from(31);

    left.set_bit(base.get_bits(0), true);
    right.set_bit(base.get_bits(0), true);

    println!("{}", left.to_string_radix(2));
    println!("{}", right.to_string_radix(2));

    assert_eq!(base.interval(&left, &right, 0), Integer::from(30))
  }
}
