// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! Substrate CLI library.

#![feature(async_await, await_macro, futures_api)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

pub use cli::error;
pub mod chain_spec;
mod service;

use tokio::prelude::Future;
use tokio::runtime::{Builder as RuntimeBuilder, Runtime, TaskExecutor};
pub use cli::{VersionInfo, IntoExit, NoCustom};
use substrate_service::{ServiceFactory, Roles as ServiceRoles, Components, ComponentClient};
// use state_machine::Externalities;
use primitives::storage::StorageKey;
use primitives::storage::well_known_keys;
use sr_primitives::generic::BlockId;
use sr_primitives::traits::Header;

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use log::info;

use ipfs::{Ipfs, IpfsOptions, IpfsService, Types, server::serve_ipfs,
	server::ipfs_responder, Compat, Future as Future3, FutureExt};
use cid::{Cid, ToCid};
use warp::{Filter, http::HeaderMap};

fn spawner<F: Future3<Output=()> + Send + 'static>(handle: TaskExecutor, future: F) {
	handle.spawn(Compat::new(Box::pin(
		future.map(|()| -> Result<(), ()> { Ok(()) })
	)));
}

fn run_ipfs<C>(handle: TaskExecutor, client: Arc<ComponentClient<C>>)
where
	C: Components + 'static,
	ComponentClient<C> : 'static
{

    let options = IpfsOptions::<Types>::default();
    let mut ipfs = Ipfs::<Types>::new(options);

	let addr : SocketAddr = "0.0.0.0:8081".parse().unwrap();

	let find_cid = | ext: (IpfsService<Types>, Arc<ComponentClient<C>>), t, h| {
		let (service, client) = ext;
		// attempt to lookup the default app IPFS Cid on-chain
		// at `well_known_keys::APP`
		if let Ok(best_header) = client.best_block_header() {
			if let Ok(Some(app_cid)) = client.storage(
				&BlockId::hash(best_header.hash()),
				&StorageKey(well_known_keys::APP.to_vec())
			) {
				if let Ok(cid) = &app_cid.0.to_cid() {
					// continue if found and valid
					return Ok((service, t, h, cid.clone()))
				}
			}
		}
		// 404 if this fails
		Err(warp::reject::not_found())
	};

	let map_to_ipfs = | res: (IpfsService<Types>, warp::path::Tail, HeaderMap, Cid) | {
		let (service, tail, header, cid) = res;
		ipfs_responder(service, cid, tail, header)
	};

	let f1 = ipfs.start_daemon().expect("can start ipfs");
	let f2 = ipfs.init_repo();
	let f3 = ipfs.open_repo();

	// fire up ipfs background process
	spawner(handle.clone(), async {
		await!(f1);
        await!(f2).expect("can init repo");
        await!(f3).expect("can open repo");
	});


	let service : IpfsService<Types> = Arc::new(Mutex::new(ipfs));
	let service2  = service.clone();

	let main = warp::any() // on `/`
		.map(move || (service.clone(), client.clone()))
		.and(warp::path::tail())
		.and(warp::header::headers_cloned())
		.and_then(find_cid) // lookup on-chain CiD
		.and_then(map_to_ipfs) // and serve via ipfs_responder
		.boxed();

	let ipfs_path = warp::path("ipfs").and(serve_ipfs(service2));
	let routes = warp::get2().and(ipfs_path.or(main));

	println!("Serving IPFS App at http://localhost:{:?}", addr.port());

    handle.spawn(warp::serve(routes).bind(addr));
}

/// The chain specification option.
#[derive(Clone, Debug)]
pub enum ChainSpec {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,
	/// Whatever the current runtime is, with simple Alice/Bob auths.
	LocalTestnet,
	/// The Dried Danta testnet.
	DriedDanta,
	/// Whatever the current runtime is with the "global testnet" defaults.
	StagingTestnet,
}

/// Get a chain config from a spec setting.
impl ChainSpec {
	pub(crate) fn load(self) -> Result<chain_spec::ChainSpec, String> {
		Ok(match self {
			ChainSpec::DriedDanta => chain_spec::dried_danta_config()?,
			ChainSpec::Development => chain_spec::development_config(),
			ChainSpec::LocalTestnet => chain_spec::local_testnet_config(),
			ChainSpec::StagingTestnet => chain_spec::staging_testnet_config(),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(ChainSpec::Development),
			"local" => Some(ChainSpec::LocalTestnet),
			"" | "danta" | "dried-danta" => Some(ChainSpec::DriedDanta),
			"staging" => Some(ChainSpec::StagingTestnet),
			_ => None,
		}
	}
}

fn load_spec(id: &str) -> Result<Option<chain_spec::ChainSpec>, String> {
	Ok(match ChainSpec::from(id) {
		Some(spec) => Some(spec.load()?),
		None => None,
	})
}

/// Parse command line arguments into service configuration.
pub fn run<I, T, E>(args: I, exit: E, version: cli::VersionInfo) -> error::Result<()> where
	I: IntoIterator<Item = T>,
	T: Into<std::ffi::OsString> + Clone,
	E: IntoExit,
{
	cli::parse_and_execute::<service::Factory, NoCustom, NoCustom, _, _, _, _, _>(
		load_spec, &version, "substrate-node", args, exit,
		|exit, _custom_args, config| {
			info!("{}", version.name);
			info!("  version {}", config.full_version());
			info!("  by Parity Technologies, 2017-2019");
			info!("Chain specification: {}", config.chain_spec.name());
			info!("Node name: {}", config.name);
			info!("Roles: {:?}", config.roles);
			let runtime = RuntimeBuilder::new().name_prefix("main-tokio-").build()
				.map_err(|e| format!("{:?}", e))?;
			let executor = runtime.executor();
			match config.roles {
				ServiceRoles::LIGHT => run_until_exit(
					runtime,
					service::Factory::new_light(config, executor).map_err(|e| format!("{:?}", e))?,
					exit
				),
				_ => run_until_exit(
					runtime,
					service::Factory::new_full(config, executor).map_err(|e| format!("{:?}", e))?,
					exit
				),
			}.map_err(|e| format!("{:?}", e))
		}
	).map_err(Into::into).map(|_| ())
}

fn run_until_exit<T, C, E>(
	mut runtime: Runtime,
	service: T,
	e: E,
) -> error::Result<()>
	where
	    T: Deref<Target=substrate_service::Service<C>>,
		C: substrate_service::Components,
		E: IntoExit,
{
	let (exit_send, exit) = exit_future::signal();

	let executor = runtime.executor();
	let client = (&service.client()).clone();
	run_ipfs::<C>(executor.clone(), client);
	cli::informant::start(&service, exit.clone(), executor.clone());

	let _ = runtime.block_on(e.into_exit());
	exit_send.fire();

	// we eagerly drop the service so that the internal exit future is fired,
	// but we need to keep holding a reference to the global telemetry guard
	let _telemetry = service.telemetry();
	drop(service);

	// TODO [andre]: timeout this future #1318
	let _ = runtime.shutdown_on_idle().wait();

	Ok(())
}
