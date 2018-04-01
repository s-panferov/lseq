use std::collections::HashMap;
use rug::rand::RandState;
use rug::Integer;

use super::base::{BitBase, DoubleBase};
use super::strategy::LSEQStrategy;
use super::ident::{Ident};
use super::meta::{Meta};

pub trait IdentGenerator<M: Meta> {
  fn generate(&mut self, replica: M, left: Option<&Ident<M>>, right: Option<&Ident<M>>)
    -> Ident<M>;
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

  fn pick_interval<M: Meta>(&self, left: Option<&Ident<M>>, right: Option<&Ident<M>>) -> (Integer, u32, Integer, Integer) {
    let mut interval = Integer::from(0);
    let mut depth: u32 = 0;

    let left_v = left.map(|i| i.digit.clone()).unwrap_or_else(|| {
      let mut int = Integer::from(0);
      int.set_bit(self.base.get_bits(depth), true);
      int
    });

    let right_v = right.map(|i| i.digit.clone()).unwrap_or_else(|| {
      let mut int = self.base.get_max(0);
      int.set_bit(self.base.get_bits(depth), true);
      int
    });

    // {
    //   println!("Left: {:?} [{}]", self.base.split(&left_v), left_v.to_string_radix(2));
    //   println!("Right: {:?} [{}]", self.base.split(&right_v), right_v.to_string_radix(2));
    // }

    while interval < 1 {
      interval = self.base.interval(&left_v, &right_v, depth);
      depth += 1;
    }

    depth -= 1;

    let step = interval
      .clone()
      .min(Integer::from(self.boundary))
      .max(Integer::from(1));

    let mut rand = RandState::new();
    let delta = step.clone().random_below(&mut rand) + 1;

    (delta, depth, left_v, right_v)
  }
}

impl<B: BitBase, M: Meta> IdentGenerator<M> for LSEQGenerator<B> {
  fn generate(
    &mut self,
    replica: M,
    left: Option<&Ident<M>>,
    right: Option<&Ident<M>>,
  ) -> Ident<M> {
    let (delta, depth, left_v, right_v) = self.pick_interval(left, right);

    self.ensure_strategy(depth);
    let strategy = self.get_strategy(depth);

    let res = match strategy {
      LSEQStrategy::AddFromLeft => {
        let left_n = self.base.normalize(left_v, depth);
        Ident::new(replica.clone(), left_n + delta)
      }
      LSEQStrategy::SubtractFromRight => {
        let right_n = self.base.normalize(right_v, depth);
        Ident::new(replica.clone(), right_n - delta)
      }
    };

    // println!("Result {}", res.debug(&self.base));
    // println!("---");

    res
  }
}

#[cfg(test)]
mod tests {
  use uuid::Uuid;
  use skiplist::OrderedSkipList;
  use rand::{thread_rng, Rng};
  use average::{Max, Min, Mean};

  use super::{DoubleBase, Ident, IdentGenerator, LSEQGenerator};

  #[test]
  fn it_works() {
    let base = DoubleBase::new(None);
    let mut gen = LSEQGenerator::new(base, None);
    let replica = Uuid::new_v4();

    let mut list = OrderedSkipList::<Ident<Uuid>>::new();
    let mut rng = thread_rng();

    for _i in 0..1000 {
      let len = list.len();
      let n = if len > 0 { rng.gen_range(0, len) } else { 0 };

      let ident = {
        let left = list.get(&(n));
        let right = list.get(&(n + 1));
        gen.generate(replica, left, right)
      };

      list.insert(ident);
    }

    let max: Max = list.iter().map(|i| i.digit.significant_bits().into()).collect();
    let min: Min = list.iter().map(|i| i.digit.significant_bits().into()).collect();
    let mean: Mean = list.iter().map(|i| i.digit.significant_bits().into()).collect();

    println!("Max: {}, Min: {}, Mean: {}", max.max(), min.min(), mean.mean());

    // list.iter().for_each(|i| println!("{:?}", i.debug(&gen.base)));
  }
  
}
