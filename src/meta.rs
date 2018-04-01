use std::fmt;

pub trait Meta: Ord + Clone + Eq + fmt::Debug {}
impl<R> Meta for R
where
  R: Ord + Clone + Eq + fmt::Debug,
{
}