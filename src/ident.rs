use std::cmp::Ordering;
use std::fmt;
use rug::Integer;

use super::base::{BitBase};
use super::meta::{Meta};

#[derive(Clone, PartialEq, Eq)]
pub struct Ident<M: Meta> {
  pub meta: M,
  pub digit: Integer,
}

impl<M: Meta> fmt::Debug for Ident<M> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut digit = self.digit.clone();
    digit.set_bit(self.digit.significant_bits() - 1, false);
    write!(f, "[Ident meta={:?} digit={}]", self.meta, digit)
  }
}

impl<M: Meta> Ident<M> {
  pub fn new(meta: M, digit: Integer) -> Ident<M> {
    Ident { meta, digit }
  }

  pub fn debug(&self, base: &BitBase) -> String {
    format!(
      "[Ident meta={:?} digit={:?}]",
      self.meta,
      base.split(&self.digit)
    )
  }
}

impl<M: Meta> PartialOrd for Ident<M> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<M: Meta> Ord for Ident<M> {
  fn cmp(&self, other: &Ident<M>) -> Ordering {
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

    let cmp_meta = self.meta.cmp(&other.meta);
    if cmp_meta != Ordering::Equal {
      return cmp_meta;
    }

    return self_length.cmp(&other_length);
  }
}