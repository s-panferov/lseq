use std::cmp::Ordering;
use std::collections::HashMap;
use rug::rand::RandState;
use std::fmt;

use rug::Integer;

pub use super::base::{BitBase, DoubleBase};
use super::strategy::LSEQStrategy;

pub trait Replica: Ord + Clone + Eq + fmt::Debug {}
impl<R> Replica for R
where
  R: Ord + Clone + Eq + fmt::Debug,
{
}

#[derive(Clone, PartialEq, Eq)]
pub struct Ident<R: Replica> {
  replica: R,
  digit: Integer,
}

impl<R: Replica> fmt::Debug for Ident<R> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut digit = self.digit.clone();
    digit.set_bit(self.digit.significant_bits() - 1, false);
    write!(f, "Ident {:?} {}", self.replica, digit)
  }
}

impl<R: Replica> Ident<R> {
  pub fn new(replica: R, mut digit: Integer, size: u32) -> Ident<R> {
    digit.set_bit(size, true);
    Ident { replica, digit }
  }

  pub fn debug_base(&self, base: &BitBase) -> String {
    let digit = self.digit.clone();
    let size = self.digit.significant_bits() - 1;
    let mut depth = 0;
    while size != base.get_bits(depth) {
      depth += 1;
    }

    println!("digit {} {} {}", digit, digit.to_string_radix(2), size);

    let mut components = vec![];
    let mut string = digit.to_string_radix(2)[1..].to_string();
    let mut skip = 1;
    for i in 0..(depth + 1) {
      let size = base.get_bits(i) - skip;
      let clone = string.clone();
      let (front, rest) = clone.split_at(size as usize + 1);
      string = rest.to_string();
      skip += size + 1;
      components.push(Integer::from(Integer::parse_radix(front, 2).unwrap()))
    }

    format!(
      "Ident {:?} {:?} {:?}",
      self.replica,
      self.digit.to_string_radix(2),
      components
    )
  }
}

impl<R: Replica> PartialOrd for Ident<R> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<R: Replica> Ord for Ident<R> {
  fn cmp(&self, other: &Ident<R>) -> Ordering {
    let self_length = self.digit.significant_bits();
    let other_length = other.digit.significant_bits();

    let self_norm: Integer;
    let other_norm: Integer;

    if self_length > other_length {
      // mine > other (in size)
      other_norm = other.digit.clone() << (self_length - other_length);
      self_norm = self.digit.clone();
    } else {
      other_norm = other.digit.clone();
      self_norm = self.digit.clone() << (other_length - self_length);
    }

    let cmp_n = self_norm.cmp(&other_norm);
    if cmp_n != Ordering::Equal {
      return cmp_n;
    }

    let cmp_replica = self.replica.cmp(&other.replica);
    if cmp_replica != Ordering::Equal {
      return cmp_replica;
    }

    return self_length.cmp(&other_length);
  }
}

pub trait IdentGenerator<R: Replica> {
  fn generate(&mut self, replica: R, left: Option<&Ident<R>>, right: Option<&Ident<R>>)
    -> Ident<R>;
}

pub struct LSEQGenerator<B: BitBase = DoubleBase> {
  base: B,
  boundary: u32,
  strategies: HashMap<usize, LSEQStrategy>,
}

impl<B: BitBase> LSEQGenerator<B> {
  pub fn new<T: Into<Option<u32>>>(base: B, boundary: T) -> LSEQGenerator<B> {
    LSEQGenerator {
      base: base,
      boundary: boundary.into().unwrap_or(10),
      strategies: HashMap::new(),
    }
  }

  fn get_strategy(&self, depth: u32) -> LSEQStrategy {
    let strategy = self.strategies.get(&(depth as usize));
    return strategy.unwrap().clone();
  }

  fn ensure_strategy(&mut self, depth: u32) {
    if self.strategies.get(&(depth as usize)).is_none() {
      let random = LSEQStrategy::random();
      self.strategies.insert(depth as usize, random.clone());
    }
  }
}

impl<B: BitBase, R: Replica> IdentGenerator<R> for LSEQGenerator<B> {
  fn generate(
    &mut self,
    replica: R,
    left: Option<&Ident<R>>,
    right: Option<&Ident<R>>,
  ) -> Ident<R> {
    let mut interval = Integer::from(0);
    let mut depth: u32 = 0;

    let first = Ident::new(replica.clone(), Integer::from(0), self.base.get_bits(0));
    let last = Ident::new(replica.clone(), self.base.get_max(0), self.base.get_bits(0));

    let left_v = left.unwrap_or(&first);
    let right_v = right.unwrap_or(&last);

    while interval < 1 {
      interval = self.base.interval(&left_v.digit, &right_v.digit, depth);
      depth += 1;
    }

    depth -= 1;

    let step = interval
      .clone()
      .min(Integer::from(self.boundary))
      .max(Integer::from(1));

    self.ensure_strategy(depth);
    let strategy = self.get_strategy(depth);
    let base_len = self.base.get_bits(depth);

    match strategy {
      LSEQStrategy::AddFromLeft => {
        // #1 Truncate tail or add bits
        let prev_bit_count = left_v.digit.significant_bits() - 1;

        let left_n = if prev_bit_count >= base_len {
          left_v.digit.clone() >> (prev_bit_count - base_len)
        } else {
          left_v.digit.clone() << (base_len - prev_bit_count)
        };

        let mut rand = RandState::new();
        let plus = step.clone().random_below(&mut rand) + 1;

        Ident::new(replica.clone(), left_n + plus, self.base.get_bits(depth))
      }
      LSEQStrategy::SubtractFromRight => {
        // #1 Truncate tail or add bits
        let prev_bit_count = right_v.digit.significant_bits() - 1;

        let right_n = if prev_bit_count >= base_len {
          right_v.digit.clone() >> (prev_bit_count - base_len)
        } else {
          right_v.digit.clone() << (base_len - prev_bit_count)
        };

        let mut rand = RandState::new();
        let minus = step.clone().random_below(&mut rand) + 1;

        Ident::new(replica.clone(), right_n - minus, self.base.get_bits(depth))
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use uuid::Uuid;
  use skiplist::OrderedSkipList;
  use super::{DoubleBase, Ident, IdentGenerator, LSEQGenerator};
  use rand::{thread_rng, Rng};

  #[test]
  fn it_works() {
    let base = DoubleBase::new(None);
    let mut gen = LSEQGenerator::new(base, None);
    let replica = Uuid::new_v4();

    let mut list = OrderedSkipList::<Ident<Uuid>>::new();
    let mut rng = thread_rng();

    for _i in 0..100 {
      let len = list.len();
      let n = if len > 0 { rng.gen_range(0, len) } else { 0 };

      let ident = {
        let left = list.get(&(n));
        let right = list.get(&(n + 1));
        gen.generate(replica, left, right)
      };

      list.insert(ident);
    }
  }
}
