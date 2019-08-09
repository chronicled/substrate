
// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

#![cfg_attr(not(feature = "std"), no_std)]
#[warn(missing_docs)]

///! A crate which contains primitives that are useful for implementation that uses staking
///! approaches in general. Definitions related to sessions, slashing, etc go here.

use codec::{Encode, Decode};
use rstd::vec::Vec;

pub mod offence;

/// Simple index type with which we can count sessions.
pub type SessionIndex = u32;

pub trait CurrentElectedSet<ValidatorId> {
	/// Returns the validator ids for the currently elected validator set.
	fn current_elected_set() -> Vec<ValidatorId>;
}

/// Proof of ownership of a specific key.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct Proof {
	/// Session index this proof is for.
	pub session: SessionIndex,
	/// A list of trie nodes.
	pub trie_nodes: Vec<Vec<u8>>,
}
