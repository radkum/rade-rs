#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod event;
mod match_;
mod prelude;
mod rade_engine;
mod rule_set;
mod utils;

// Public API - only export what external users need
pub use event::{Event, EventSerialized, Events};
pub use match_::{MatchedRules, Matches};
pub use rade_engine::RadeEngine;
// Crate-internal re-exports (not visible to external users)
pub(crate) use rule_set::{Comparator, OperandContainer, ResultMap, Val};
pub use rule_set::{Rule, RuleSet, RuleSetError, Rules};
pub(crate) use utils::{FatString, InsensitiveFlag, RadeResult};

type Guid = uuid::Uuid;
