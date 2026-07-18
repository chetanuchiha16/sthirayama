use std::fmt::Debug;

use bitcode::{DecodeOwned, Encode};

pub trait TypeSkipListKey: Encode + DecodeOwned + Clone + PartialOrd + Debug {}

impl<K: Encode + DecodeOwned + Clone + PartialOrd + Debug> TypeSkipListKey for K {}
pub trait TypeSkipListValue: Encode + DecodeOwned + Debug + Clone {}

impl<V: Encode + DecodeOwned + Debug + Clone> TypeSkipListValue for V {}
