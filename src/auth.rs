pub use self::Auth::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Auth {
    Anyone,
    HalfOp,
    Op,
    Owner
}
