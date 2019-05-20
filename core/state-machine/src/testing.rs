// Copyright 2017-2019 Parity Technologies (UK) Ltd.
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

//! Test implementation for Externalities.

use std::collections::{HashMap, BTreeMap};
use std::iter::FromIterator;
use std::marker::PhantomData;
use hash_db::Hasher;
use crate::backend::{InMemory, Backend};
use crate::{NeverOffchainExt, Ext};
use primitives::storage::well_known_keys::is_child_storage_key;
use crate::changes_trie::{
	compute_changes_trie_root, InMemoryStorage as ChangesTrieInMemoryStorage, AnchorBlockId
};
use primitives::storage::well_known_keys::{CHANGES_TRIE_CONFIG, CODE, HEAP_PAGES};
use parity_codec::Encode;
use super::{ChildStorageKey, Externalities, OverlayedChanges};

const EXT_NOT_ALLOWED_TO_FAIL: &str = "Externalities not allowed to fail within runtime";

/// Simple HashMap-based Externalities impl.
pub struct TestExternalities<H: Hasher> {
	overlay: OverlayedChanges,
	backend: InMemory<H>,
	changes_trie_storage: ChangesTrieInMemoryStorage<H>,
	_hasher: PhantomData<H>,
}

impl<H: Hasher> TestExternalities<H> {
	/// Create a new instance of `TestExternalities`
	pub fn new(inner: HashMap<Vec<u8>, Vec<u8>>) -> Self {
		Self::new_with_code(&[], inner)
	}

	/// Create a new instance of `TestExternalities`
	pub fn new_with_code(code: &[u8], mut inner: HashMap<Vec<u8>, Vec<u8>>) -> Self {
		let mut overlay = OverlayedChanges::default();

		super::set_changes_trie_config(
			&mut overlay,
			inner.get(&CHANGES_TRIE_CONFIG.to_vec()).cloned(),
			false,
		).expect("changes trie configuration is correct in test env; qed");

		inner.insert(HEAP_PAGES.to_vec(), 8u64.encode());
		inner.insert(CODE.to_vec(), code.to_vec());

		TestExternalities {
			overlay,
			changes_trie_storage: ChangesTrieInMemoryStorage::new(),
			backend: inner.into(),
			_hasher: Default::default(),
		}
	}

	/// Insert key/value into backend
	pub fn insert(&mut self, k: Vec<u8>, v: Vec<u8>) {
		self.backend = self.backend.update(vec![(None, k, Some(v))]);
	}

	/// Iter to all pairs in key order
	pub fn iter_pairs_in_order(&self) -> impl Iterator<Item=(Vec<u8>, Vec<u8>)> {
		self.backend.pairs().iter()
			.map(|&(ref k, ref v)| (k.to_vec(), Some(v.to_vec())))
			.chain(self.overlay.committed.top.clone().into_iter().map(|(k, v)| (k, v.value)))
			.chain(self.overlay.prospective.top.clone().into_iter().map(|(k, v)| (k, v.value)))
			.collect::<BTreeMap<_, _>>()
			.into_iter()
			.filter_map(|(k, maybe_val)| maybe_val.map(|val| (k, val)))
	}
}

impl<H: Hasher> TestExternalities<H> where H::Out: Ord {
	/// Return externalities
	pub fn ext(&mut self) -> Ext<H, InMemory<H>, ChangesTrieInMemoryStorage<H>, NeverOffchainExt> {
		Ext::new(&mut self.overlay, &self.backend, Some(&self.changes_trie_storage), None)
	}
}

impl<H: Hasher> ::std::fmt::Debug for TestExternalities<H> {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "overlay: {:?}\nbackend: {:?}", self.overlay, self.backend.pairs())
	}
}

impl<H: Hasher> PartialEq for TestExternalities<H> {
	/// This doesn't test if they are in the same state, only if they contains the
	/// same data at this state
	fn eq(&self, other: &TestExternalities<H>) -> bool {
		self.iter_pairs_in_order().eq(other.iter_pairs_in_order())
	}
}

impl<H: Hasher> FromIterator<(Vec<u8>, Vec<u8>)> for TestExternalities<H> {
	fn from_iter<I: IntoIterator<Item=(Vec<u8>, Vec<u8>)>>(iter: I) -> Self {
		let mut t = Self::new(Default::default());
		t.backend = t.backend.update(iter.into_iter().map(|(k, v)| (None, k, Some(v))).collect());
		t
	}
}

impl<H: Hasher> Default for TestExternalities<H> {
	fn default() -> Self { Self::new(Default::default()) }
}

impl<H: Hasher> From<TestExternalities<H>> for HashMap<Vec<u8>, Vec<u8>> {
	fn from(tex: TestExternalities<H>) -> Self {
		tex.iter_pairs_in_order().collect()
	}
}

impl<H: Hasher> From<HashMap<Vec<u8>, Vec<u8>>> for TestExternalities<H> {
	fn from(hashmap: HashMap<Vec<u8>, Vec<u8>>) -> Self {
		Self::from_iter(hashmap)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use primitives::{Blake2Hasher, H256};
	use hex_literal::hex;

	#[test]
	fn commit_should_work() {
		let mut test_ext = TestExternalities::<Blake2Hasher>::default();
		let mut ext = test_ext.ext();
		ext.set_storage(b"doe".to_vec(), b"reindeer".to_vec());
		ext.set_storage(b"dog".to_vec(), b"puppy".to_vec());
		ext.set_storage(b"dogglesworth".to_vec(), b"cat".to_vec());
		const ROOT: [u8; 32] = hex!("cc65c26c37ebd4abcdeb3f1ecd727527051620779a2f6c809bac0f8a87dbb816");
		assert_eq!(ext.storage_root(), H256::from(ROOT));
	}

	#[test]
	fn set_and_retrieve_code() {
		let mut test_ext = TestExternalities::<Blake2Hasher>::default();
		let mut ext = test_ext.ext();

		let code = vec![1, 2, 3];
		ext.set_storage(CODE.to_vec(), code.clone());

		assert_eq!(&ext.storage(CODE).unwrap(), &code);
	}
}
