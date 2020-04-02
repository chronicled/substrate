// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! In-memory implementation of offchain workers database.

use std::collections::hash_map::{HashMap, Entry};
use crate::offchain::OffchainStorage;
use std::iter::Iterator;

/// In-memory storage for offchain workers.
#[derive(Debug, Clone, Default)]
pub struct InMemOffchainStorage {
	storage: HashMap<Vec<u8>, Vec<u8>>,
}

impl InMemOffchainStorage {
	/// Consume the offchain storage and iterate over all key value pairs.
	pub fn into_iter(self) -> impl Iterator<Item=(Vec<u8>,Vec<u8>)> {
		self.storage.into_iter()
	}

	/// Iterate over all key value pairs by reference.
	pub fn iter<'a>(&'a self) -> impl Iterator<Item=(&'a Vec<u8>,&'a Vec<u8>)> {
		self.storage.iter()
	}

	/// Remove a key and it's associated value from the offchain database.
	pub fn remove(&mut self, prefix: &[u8], key: &[u8]) {
		let key: Vec<u8> = prefix.iter().chain(key).cloned().collect();
		let _ = self.storage.remove(&key);
	}
}

impl OffchainStorage for InMemOffchainStorage {
	fn set(&mut self, prefix: &[u8], key: &[u8], value: &[u8]) {
		let key = prefix.iter().chain(key).cloned().collect();
		self.storage.insert(key, value.to_vec());
	}

	fn get(&self, prefix: &[u8], key: &[u8]) -> Option<Vec<u8>> {
		let key: Vec<u8> = prefix.iter().chain(key).cloned().collect();
		self.storage.get(&key).cloned()
	}

	fn compare_and_set(
		&mut self,
		prefix: &[u8],
		key: &[u8],
		old_value: Option<&[u8]>,
		new_value: &[u8],
	) -> bool {
		let key = prefix.iter().chain(key).cloned().collect();

		match self.storage.entry(key) {
			Entry::Vacant(entry) => if old_value.is_none() {
				entry.insert(new_value.to_vec());
				true
			} else { false },
			Entry::Occupied(ref mut entry) if Some(entry.get().as_slice()) == old_value => {
				entry.insert(new_value.to_vec());
				true
			},
			_ => false,
		}
	}
}




/// Change to be applied to the offchain worker db in regards to a key.
#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub enum OffchainOverlayedChange {
	/// Remove the data associated with the key
	Remove,
	/// Overwrite the value of an associated key
	SetValue(Vec<u8>),
}

/// In-memory storage for offchain workers.
#[derive(Debug, Clone)]
pub struct OffchainOverlayedChanges {
	storage: HashMap<Vec<u8>, OffchainOverlayedChange>,
}

/// Create an empty set of overlay changes.
///
/// The `Default` impl has to be done manually to avoid requirement of
/// a `Default` impl for `OffchainOverlayedChange`
impl Default for OffchainOverlayedChanges {
	fn default() -> Self {
		Self {
			storage: HashMap::new()
		}
	}
}


impl OffchainOverlayedChanges {
	/// Consume the offchain storage and iterate over all key value pairs.
	pub fn into_iter(self) -> impl Iterator<Item=(Vec<u8>,OffchainOverlayedChange)> {
		self.storage.into_iter()
	}

	/// Iterate over all key value pairs by reference.
	pub fn iter<'a>(&'a self) -> impl Iterator<Item=(&'a Vec<u8>,&'a OffchainOverlayedChange)> {
		self.storage.iter()
	}

	/// Drain all elements of changeset.
	pub fn drain<'a,'d>(&'a mut self) -> impl Iterator<Item=(Vec<u8>, OffchainOverlayedChange)> + 'd where 'a : 'd {
		self.storage.drain()
	}

	/// Remove a key and it's associated value from the offchain database.
	pub fn remove(&mut self, prefix: &[u8], key: &[u8]) {
		let key: Vec<u8> = prefix.iter().chain(key).cloned().collect();
		let _ = self.storage.insert(key, OffchainOverlayedChange::Remove);
	}

	/// Set the value associated with a key under a prefix to the value provided.
	pub fn set(&mut self, prefix: &[u8], key: &[u8], value: &[u8]) {
		let key = prefix.iter().chain(key).cloned().collect();
		self.storage.insert(key, OffchainOverlayedChange::SetValue(value.to_vec()));
	}
}