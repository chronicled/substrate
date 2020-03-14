// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::{io, io::ErrorKind};

use kvdb::{DBOp, DBTransaction, DBValue, KeyValueDB};

/// Database configuration
#[derive(Clone)]
pub struct DatabaseConfig {
	pub columns: u32,
}

pub type KeyValuePair = (Box<[u8]>, Box<[u8]>);

impl DatabaseConfig {
	/// Create new `DatabaseConfig` with default parameters and specified set of columns.
	/// Note that cache sizes must be explicitly set.
	///
	/// # Safety
	///
	/// The number of `columns` must not be zero.
	pub fn with_columns(columns: u32) -> Self {
		assert!(columns > 0, "the number of columns must not be zero");
		Self { columns }
	}
}

/// Key-Value database.
pub struct Database {
	_db: sled::Db,
	trees: Vec<sled::Tree>,
}

fn to_io_err<E: std::fmt::Debug>(e: E) -> io::Error {
	io::Error::new(ErrorKind::Other, format!("{:?}", e))
}

impl parity_util_mem::MallocSizeOf for Database {
	fn size_of(&self, _ops: &mut parity_util_mem::MallocSizeOfOps) -> usize {
		0
	}
}


impl Database {
	/// Open database file. Creates if it does not exist.
	///
	/// # Safety
	///
	/// The number of `config.columns` must not be zero.
	pub fn open(config: &DatabaseConfig, path: &str) -> io::Result<Database> {
		assert!(config.columns > 0, "the number of columns must not be zero");

		let db = sled::Config::default()
			.path(path)
			.cache_capacity(64 * 1024 * 1024)
			.use_compression(true)
			.open().map_err(to_io_err)?;

		let trees = (0 .. config.columns).map(|c| {
			let name = format!("col{}", c);
			db.open_tree(name.as_bytes()).unwrap()
		}).collect();

		Ok(Database {
			_db: db,
			trees,
		})
	}

	/// Helper to create new transaction for this database.
	pub fn transaction(&self) -> DBTransaction {
		DBTransaction::new()
	}

	/// Commit transaction to database.
	pub fn write(&self, tr: DBTransaction) -> io::Result<()> {
		use sled::transaction::Transactional;

		let ops = tr.ops;

		self.trees[..].transaction(move |trees| {
			for op in &ops {
				match op {
					DBOp::Insert { col: c, key, value } => {
						trees[*c as usize].insert(key.to_vec(), value.to_vec())?;
					}
					DBOp::Delete { col: c, key } => {
						trees[*c as usize].remove(key.to_vec())?;
					}
				}
			}
			Ok(()) as sled::transaction::ConflictableTransactionResult<(), ()>
		}).map_err(to_io_err)?;
		Ok(())
	}

	/// Get value by key.
	pub fn get(&self, col: u32, key: &[u8]) -> io::Result<Option<DBValue>> {
		self.trees[col as usize].get(key).map(|r| r.map(|v| DBValue::from(v.to_vec()))).map_err(to_io_err)
	}

	/// Get value by partial key. Prefix size should match configured prefix size. Only searches flushed values.
	pub fn get_by_prefix(&self, col: u32, prefix: &[u8]) -> Option<Box<[u8]>> {
		self.iter_from_prefix(col, prefix).next().map(|(_, v)| v)
	}

	fn iter_from_prefix<'a>(&'a self, col: u32, prefix: &'a [u8]) -> impl Iterator<Item = KeyValuePair> + 'a {
		self.trees[col as usize].scan_prefix(prefix).map(|r| {
			let (k, v) = r.unwrap();
			(k.to_vec().into_boxed_slice(), v.to_vec().into_boxed_slice())
		})
	}

	pub fn num_columns(&self) -> u32 {
		self.trees.len() as u32
	}
}

// duplicate declaration of methods here to avoid trait import in certain existing cases
// at time of addition.
impl KeyValueDB for Database {
	fn get(&self, col: u32, key: &[u8]) -> io::Result<Option<DBValue>> {
		Database::get(self, col, key)
	}

	fn get_by_prefix(&self, col: u32, prefix: &[u8]) -> Option<Box<[u8]>> {
		Database::get_by_prefix(self, col, prefix)
	}

	fn write_buffered(&self, _transaction: DBTransaction) {
		unimplemented!()
	}

	fn write(&self, transaction: DBTransaction) -> io::Result<()> {
		Database::write(self, transaction)
	}

	fn flush(&self) -> io::Result<()> {
		Ok(())
	}

	fn iter<'a>(&'a self, col: u32) -> Box<dyn Iterator<Item = KeyValuePair> + 'a> {
		let unboxed = self.trees[col as usize].iter().map(|r| {
			let (k, v) = r.unwrap();
			(k.to_vec().into_boxed_slice(), v.to_vec().into_boxed_slice())
		});
		Box::new(unboxed.into_iter())
	}

	fn iter_from_prefix<'a>(&'a self, col: u32, prefix: &'a [u8]) -> Box<dyn Iterator<Item = KeyValuePair> + 'a> {
		let unboxed = Database::iter_from_prefix(self, col, prefix);
		Box::new(unboxed.into_iter())
	}

	fn restore(&self, _new_db: &str) -> io::Result<()> {
		unimplemented!()
	}

	fn io_stats(&self, _kind: kvdb::IoStatsKind) -> kvdb::IoStats {
		kvdb::IoStats::empty()
	}
}

