/// A runtime module for managing rotating validator sets

/// This module allows any root call to add a single validator to the set.
/// The addition takes effect at the end of a session. Sessions are 10 blocks.

// Things that could be added to make this cooler:
// * Adding multiple validators per session
// * Removing validators
// * Events
// * Loosely couple to session module?


use support::{decl_module, decl_storage, /*decl_event,*/ dispatch::Result};
use system::ensure_root;
use session::{ OnSessionEnding, SelectInitialValidators };
//TODO Why is this part of staking primitives. It feels more session-y.
use sr_staking_primitives::SessionIndex;
use rstd::vec::Vec;

/// The module's configuration trait.
//TODO For now I'm tightly coupling to the session module
pub trait Trait: system::Trait + session::Trait {
	// The overarching event type.
	//type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as AddValidator {
		// Stores possible validator to be added to the set next time
		QueuedValidator: Option<T::AccountId>;
	}
}

// The module's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		//fn deposit_event() = default;

		// Queue a new validator to be added at the next session end.
		pub fn queue_validator(origin, n00b: T::AccountId) -> Result {

			let _ = ensure_root(origin)?;

			// TODO check that you aren't overwriting another validaotr first
			// Write the new validator to storage
			<QueuedValidator<T>>::put(n00b);

			// TODO Event
			// Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}
	}
}

// Not Actually Used
// decl_event!(
// 	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
// 		SomethingStored(u32, AccountId),
// 	}
// );

impl<T: Trait> OnSessionEnding<T::AccountId> for Module<T> {
	fn on_session_ending(_ending_index: SessionIndex, _will_apply_at: SessionIndex) -> Option<Vec<T::AccountId>> {
		match <QueuedValidator<T>>::get() {
			Some(n00b) => {
				// Get the list of current validators from the session module
				let mut validators = session::Module::<T>::validators();
				validators.push(n00b);
				Some(validators)
			}
			None => None
		}

	}
}

impl<T: Trait> SelectInitialValidators<T::AccountId> for Module<T> {
	fn select_initial_validators() -> Option<Vec<T::AccountId>> {
		// From https://crates.parity.io/pallet_session/trait.SelectInitialValidators.html
		// If None is returned all accounts that have session keys set in the genesis block will be validators.
		// So I think I'm just telling it to read the initial validators from
		// The genesis config. But what field? Is there a field for tsession module?
		None
	}
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use primitives::H256;
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, weights::Weight, Perbill,
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl Trait for Test {
		type Event = ();
	}
	type TemplateModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn it_works_for_default_value() {
		new_test_ext().execute_with(|| {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}
}
