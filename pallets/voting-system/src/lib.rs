#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::inherent::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	// pub enum ElectionPhase {
	// 	None,
	// 	Initialization,
	// 	Registration,
	// 	BiasedSigner,
	// 	Voting,
	// 	Counting,
	// }

	#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
	#[scale_info(skip_type_params(T))]
	pub struct State<T: Config> {
		central_authority: T::AccountId,
		// phase: ElectionPhase
	}

	#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Voter<T: Config> {
		account_id: T::AccountId
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn state)]
	pub type Election<T> = StorageValue<_, State<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Entered Initialisation Phase
		ElectionInitialisationStarted { when: T::BlockNumber },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn change_phase(origin: OriginFor<T>) -> DispatchResult {
			// make sure that it is signed by the CA

			// change the phase

			// Emit an event.
			Ok(())
		}
	}
}
