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

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub central_authority: Option<T::AccountId>,
		pub voters: Vec<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { central_authority: None, voters: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			if let Some(ref ca) = self.central_authority {
				CentralAuthority::<T>::put(ca);
			}
			for voter in &self.voters {
				Voters::<T>::insert(voter, 0);
			}
		}
	}

	#[cfg(feature = "std")]
	impl<T: Config> GenesisConfig<T> {
		/// Direct implementation of `GenesisBuild::build_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
			<Self as GenesisBuild<T>>::build_storage(self)
		}

		/// Direct implementation of `GenesisBuild::assimilate_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
			<Self as GenesisBuild<T>>::assimilate_storage(self, storage)
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum ElectionPhase {
		None,
		Initialization,
		Registration,
		BiasedSigner,
		Voting,
		Counting,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn ca)]
	pub type CentralAuthority<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn phase)]
	pub type Phase<T: Config> = StorageValue<_, ElectionPhase, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn voters)]
	pub type Voters<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u32, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Phase changed
		PhaseChanged { when: T::BlockNumber, phase: ElectionPhase },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Internal error
		InternalError,
		/// Error when the sender is not CA
		SenderNotCA,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(0)]
		pub fn change_phase(origin: OriginFor<T>, phase: ElectionPhase) -> DispatchResult {
			// make sure that it is signed by the CA
			let sender = ensure_signed(origin)?;
			let ca = Self::ca();
			if let Some(ca) = ca {
				ensure!(sender == ca, <Error<T>>::SenderNotCA);
			} else {
				// if CA is not set, return error
				return Err(Error::<T>::InternalError.into())
			}

			// ensure!(sender == ca, <Error<T>>::SenderNotCA);

			<Phase<T>>::put(phase.clone());
			Self::deposit_event(Event::PhaseChanged {
				when: frame_system::Pallet::<T>::block_number(),
				phase,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_phase() -> Option<ElectionPhase> {
			<Phase<T>>::get()
		}
	}
}
