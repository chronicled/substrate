// Copyright 2018 Parity Technologies (UK) Ltd.
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

//! Memory tracking utilities

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

use parity_util_mem::{MallocSizeOf, MallocSizeOfExt};
pub use parity_util_mem::MallocSizeOfOps;
pub use sp_memory_derive::*;

pub trait HeapSize {
    fn heap_size(&self, ops: &mut MallocSizeOfOps) -> usize;
}

impl<T: MallocSizeOf> HeapSize for T {
    fn heap_size(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.size_of(ops)
    }
}

#[cfg(feature = "std")]
/// Trait that identifies that object can track its memory footprint (when in std).
pub trait MaybeHeapSize: HeapSize {}
#[cfg(feature = "std")]
impl<T: HeapSize> MaybeHeapSize for T {}
#[cfg(not(feature = "std"))]
/// Trait that identifies that object can track its memory footprint (when in std).
pub trait MaybeHeapSize {}
#[cfg(not(feature = "std"))]
impl<T> MaybeHeapSize for T {}