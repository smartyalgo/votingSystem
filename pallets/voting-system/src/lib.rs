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
	use frame_support::{inherent::Vec, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub central_authority: Option<T::AccountId>,
		pub candidates: Vec<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { central_authority: None, candidates: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			Phase::<T>::put(ElectionPhase::Initialization);

			if let Some(ref ca) = self.central_authority {
				CentralAuthority::<T>::put(ca);
			}
			for candidate in &self.candidates {
				Candidates::<T>::insert(candidate, Candidate { name: vec![] });
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
		Completed,
	}

	impl ElectionPhase {
		fn increment(&self) -> Self {
			use ElectionPhase::*;
			match *self {
				None => Initialization,
				Initialization => Registration,
				Registration => BiasedSigner,
				BiasedSigner => Voting,
				Voting => Counting,
				Counting => Completed,
				_ => None,
			}
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct Voter {
		pub blinded_pubkey: Vec<u8>,
		pub is_eligible: bool,
		pub signed_blinded_pubkey: Vec<u8>,
	}

	/// Todo: determine maximum length of struct storage
	impl MaxEncodedLen for Voter {
		fn max_encoded_len() -> usize {
			usize::MAX - 1
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct Candidate {
		pub name: Vec<u8>,
	}

	/// Todo: determine maximum length of struct storage
	impl MaxEncodedLen for Candidate {
		fn max_encoded_len() -> usize {
			usize::MAX - 1
		}
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
	#[pallet::getter(fn candidates)]
	pub type Candidates<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Candidate, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn voters)]
	pub type Voters<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Voter, OptionQuery>;

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
		/// Voter already exists
		VoterAlreadyExists,
		/// Invalid phase change
		InvalidPhaseChange,
		/// Invalid phase
		InvalidPhase,
		/// Bad Sender
		BadSender,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// fn verify_change_to_registration(origin: <T as Config>::AccountId) -> Option<Error<T>> {

		// }

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(0)]
		pub fn change_phase(origin: OriginFor<T>) -> DispatchResult {
			// make sure that it is signed by the CA
			let sender = ensure_signed(origin)?;

			let ca = Self::ca();
			if let Some(ca) = ca {
				ensure!(sender == ca, <Error<T>>::SenderNotCA);
			} else {
				// if CA is not set, return error
				return Err(Error::<T>::InternalError.into())
			}

			// Update the phase
			let new_phase = Self::phase().expect("REASON").increment();
			<Phase<T>>::put(new_phase.clone());

			// Emit event
			Self::deposit_event(Event::PhaseChanged {
				when: frame_system::Pallet::<T>::block_number(),
				phase: new_phase,
			});

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(1)]
		pub fn add_voter(
			origin: OriginFor<T>,
			voter: T::AccountId,
			blinded_pubkey: Vec<u8>,
			signed_blinded_pubkey: Vec<u8>,
			is_eligible: bool,
		) -> DispatchResult {
			// make sure that it is signed by the CA
			let sender = ensure_signed(origin)?;
			let ca = Self::ca();
			if let Some(ca) = ca {
				ensure!(sender == ca, <Error<T>>::SenderNotCA);
			} else {
				// if CA is not set, return error
				return Err(Error::<T>::InternalError.into())
			}

			// Voters can only be added during the registration phase
			ensure!(
				Self::get_phase() == Some(ElectionPhase::Registration),
				<Error<T>>::InvalidPhase
			);

			// If the voter already exists, return error
			ensure!(!<Voters<T>>::contains_key(&voter), <Error<T>>::VoterAlreadyExists);

			// Update the phase
			<Voters<T>>::insert(
				voter,
				Voter { blinded_pubkey, is_eligible, signed_blinded_pubkey },
			);

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(2)]
		pub fn update_candidate_info(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			name: Vec<u8>,
		) -> DispatchResult {
			// make sure that it is signed by the CA
			let sender = ensure_signed(origin)?;
			ensure!(sender == candidate, <Error<T>>::BadSender);

			// Update the phase
			<Candidates<T>>::insert(candidate, Candidate { name });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_phase() -> Option<ElectionPhase> {
			<Phase<T>>::get()
		}

		pub fn get_voter(voter: T::AccountId) -> Option<Voter> {
			<Voters<T>>::get(voter)
		}

		pub fn get_candidate(candidate: T::AccountId) -> Option<Candidate> {
			<Candidates<T>>::get(candidate)
		}
	}

	// impl<T: Config> Pallet<T> {
	// 	fn ensure_ca
	// }
}
