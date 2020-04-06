// Copyright 2020 Parity Technologies (UK) Ltd.
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


//! Offchain worker related configuration parameters.
//!
//! A subset of configuration parameters which are relevant to
//! the inner working of offchain workers. The usage is solely
//! targeted at handling input parameter parsing providing
//! a reasonable abstraction.

use structopt::StructOpt;
use sc_service::Configuration;

use sc_service::config::OffchainWorkerConfig;
use structopt::clap::arg_enum;


use crate::error;

#[allow(missing_docs)]
arg_enum! {
	/// Whether offchain workers are enabled or not as a whole.
	#[allow(missing_docs)]
	#[derive(Debug, Clone)]
	pub enum OffchainWorkerEnabled {
		Always,
		Never,
		WhenValidating,
	}
}

/// Offchain worker related parameters.
#[derive(Debug, StructOpt, Clone)]
pub struct OffchainWorkerParams {

	/// Should execute offchain workers on every block.
	///
	/// By default it's only enabled for nodes that are authoring new blocks.
	#[structopt(
		long = "offchain-worker",
		value_name = "ENABLED",
		possible_values = &OffchainWorkerEnabled::variants(),
		case_insensitive = true,
		default_value = "WhenValidating"
    )]
    pub enabled: OffchainWorkerEnabled,

	/// Allow access to offchain workers indexing API
	///
	/// Enables runtime to write directly to the offchain worker's
	/// DB during block import.
    #[structopt(
        long = "enable-offchain-indexing",
        value_name = "ENABLE_OFFCHAIN_INDEXING"
    )]
	pub indexing_enabled: bool,
}

impl OffchainWorkerParams {
	/// Load spec to `Configuration` from `OffchainWorkerParams` and spec factory.
	pub fn update_config<'a>(
		&self,
		mut config: &'a mut Configuration,
        role: sc_service::Roles,
	) -> error::Result<()>
	{
        let enabled = match (&self.enabled, role) {
			(OffchainWorkerEnabled::WhenValidating, sc_service::Roles::AUTHORITY) => true,
			(OffchainWorkerEnabled::Always, _) => true,
			(OffchainWorkerEnabled::Never, _) => false,
			(OffchainWorkerEnabled::WhenValidating, _) => false,
		};

        let indexing_enabled = enabled && self.indexing_enabled;

        config.offchain_worker = OffchainWorkerConfig { enabled, indexing_enabled};

        Ok(())
	}
}
