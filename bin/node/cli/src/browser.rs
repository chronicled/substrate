// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::chain_spec::ChainSpec;
use log::info;
use wasm_bindgen::prelude::*;
use browser_utils::{
	Client,
	browser_configuration, set_console_error_panic_hook, init_console_log,
};
use std::str::FromStr;

/// Starts the client.
#[wasm_bindgen]
pub async fn start_client(chain_spec: Option<String>, log_level: String) -> Result<Client, JsValue> {
	init_logging(log_level)?;
	let chain_spec = match chain_spec {
		Some(chain_spec) => ChainSpec::from_json_bytes(chain_spec.as_bytes().to_vec())
			.map_err(|e| format!("{:?}", e))?,
		None => crate::chain_spec::development_config(),
	};

	start_from_chain_spec(chain_spec).await
}

/// Starts the development client.
#[wasm_bindgen]
pub async fn start_dev_client(log_level: String) -> Result<Client, JsValue> {
	init_logging(log_level)?;
	let chain_spec = crate::chain_spec::browser_test_config();
	start_from_chain_spec(chain_spec).await
}

fn init_logging(log_level: String) -> Result<(), JsValue> {
	set_console_error_panic_hook();
	init_console_log(
		log::Level::from_str(&log_level)
			.map_err(|e| format!("{:?}", e))?
	).map_err(|e| format!("{:?}", e))?;

	Ok(())
}

async fn start_from_chain_spec(chain_spec: ChainSpec) -> Result<Client, JsValue> {
	let config = browser_configuration(chain_spec).await
		.map_err(|e| format!("{:?}", e))?;

	info!("Substrate browser node");
	info!("✌️  version {}", config.impl_version);
	info!("❤️  by Parity Technologies, 2017-2020");
	info!("📋 Chain specification: {}", config.chain_spec.name());
	info!("🏷  Node name: {}", config.network.node_name);
	info!("👤 Role: {:?}", config.role);

	// Create the service. This is the most heavy initialization step.
	let service = crate::service::new_light(config)
		.map_err(|e| format!("{:?}", e))?;

	Ok(browser_utils::start_client(service))
}
