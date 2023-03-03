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
	use frame_support::{inherent::Vec, pallet_prelude::*, traits::ValidatorRegistration};
	use frame_system::pallet_prelude::*;
	use scale_info::prelude::string::String;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

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
				Completed => Completed,
			}
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct Voter {
		pub blinded_pubkey: Vec<u8>,
		pub is_eligible: bool,
		// Signed by CA after verifying eligibility
		pub signed_blinded_pubkey: Vec<u8>,
		pub personal_data_hash: Vec<u8>,
	}

	/// Todo: determine maximum length of struct storage
	impl MaxEncodedLen for Voter {
		fn max_encoded_len() -> usize {
			usize::MAX - 1
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct Candidate {
		pub name: String,
		// RSA Key
		pub pubkey: Vec<u8>,
	}

	/// Todo: determine maximum length of struct storage
	impl MaxEncodedLen for Candidate {
		fn max_encoded_len() -> usize {
			usize::MAX - 1
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct Ballot {
		pub commitment: Vec<u8>,
		pub signature: Vec<u8>,
		pub nonce: u64,
	}
	/// Todo: determine maximum length of struct storage
	impl MaxEncodedLen for Ballot {
		fn max_encoded_len() -> usize {
			usize::MAX - 1
		}
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct BallotKey {
		pub public: Vec<u8>,
		pub private: Vec<u8>,
	}
	/// Todo: determine maximum length of struct storage
	impl MaxEncodedLen for BallotKey {
		fn max_encoded_len() -> usize {
			usize::MAX - 1
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
		type SignatureLength: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn ca)]
	// TODO: Change to Super User for controlling the phases
	pub type CentralAuthority<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	// TODO: Add array of Registers for registering voters

	#[pallet::storage]
	#[pallet::getter(fn ballot_key)]
	pub type BallotKeys<T: Config> = StorageValue<_, BallotKey, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn phase)]
	pub type Phase<T: Config> = StorageValue<_, ElectionPhase, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type Candidates<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Candidate, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn voters)]
	pub type Voters<T: Config> = StorageMap<_, Twox64Concat, u64, Voter, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn blinded_signatures)] // (voter_id, candidate_id) -> signature
	pub type BlindedSignatures<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		u64,
		Twox64Concat,
		T::AccountId,
		BoundedVec<u8, T::SignatureLength>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn voter_count)]
	pub type VoterCount<T: Config> = StorageValue<_, u64, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn ballots)]
	pub type Ballots<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Ballot, OptionQuery>;

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
		/// Ballot already exists
		BallotAlreadyExists,
		/// Ballot does not exist
		BallotNotFound,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub central_authority: Option<T::AccountId>,
		pub candidates: Vec<T::AccountId>,
		pub ballot_public_key: Vec<u8>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { central_authority: None, candidates: Vec::new(), ballot_public_key: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			Phase::<T>::put(ElectionPhase::Initialization);

			if let Some(ref ca) = self.central_authority {
				CentralAuthority::<T>::put(ca);
			}

			let pubkey = &self.ballot_public_key;
			if pubkey.len() == 0 {
				panic!("Ballot public key is empty");
			}

			BallotKeys::<T>::put(BallotKey { public: pubkey.clone(), private: Vec::new() });

			if self.candidates.len() < 2 {
				panic!("At least 2 candidates are required");
			}
			for candidate in &self.candidates {
				// pubkey with place holder
				Candidates::<T>::insert(
					candidate,
					Candidate { name: "".to_string(), pubkey: Vec::new() },
				);
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

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[pallet::call_index(0)]
		pub fn change_phase(origin: OriginFor<T>) -> DispatchResult {
			// make sure that it is signed by the CA
			let sender = ensure_signed(origin)?;

			let ca = Self::ca();
			if let Some(ca) = ca {
				ensure!(sender == ca, <Error<T>>::SenderNotCA);
			} else {
				// if CA is not set, return error
				return Err(Error::<T>::InternalError.into());
			}

			let current_phase = Self::phase();
			match Some(current_phase) {
				ElectionPhase::Registration => {
					// Check if there exist voter with incomplete signature
				},
				ElectionPhase::BiasedSigner => {
					// Check if there exist voter with incomplete signature
					if (false) {
						return Err(Error::<T>::InvalidPhaseChange.into());
					}
				},
			}

			// Update the phase
			// TODO: Refactor this section
			let new_phase = Self::phase().expect("REASON").increment();
			Phase::<T>::put(new_phase.clone());

			// Emit event
			Self::deposit_event(Event::PhaseChanged {
				when: frame_system::Pallet::<T>::block_number(),
				phase: new_phase,
			});

			Ok(())
		}

		// TODO: After system is working, experiment with removing or changing value to 0
		#[pallet::weight(0)]
		#[pallet::call_index(1)]
		pub fn add_voter(
			origin: OriginFor<T>,
			blinded_pubkey: Vec<u8>,
			signed_blinded_pubkey: Vec<u8>,
			personal_data_hash: Vec<u8>,
			is_eligible: bool,
		) -> DispatchResult {
			// make sure that it is signed by the CA
			let sender = ensure_signed(origin)?;
			// TODO: Change this to be a valid register
			let ca = Self::ca();
			if let Some(ca) = ca {
				ensure!(sender == ca, <Error<T>>::SenderNotCA);
			} else {
				// if CA is not set, return error
				// TODO: Change to Incorrect Configuration
				return Err(Error::<T>::InternalError.into());
			}

			// Voters can only be added during the registration phase
			ensure!(
				Self::get_phase() == Some(ElectionPhase::Registration),
				<Error<T>>::InvalidPhase
			);

			// Get the voter count
			let voter_count = Self::voter_count().unwrap_or(0);
			let new_voter_index = voter_count + 1;

			// Add the voter
			<Voters<T>>::insert(
				new_voter_index,
				Voter { blinded_pubkey, is_eligible, signed_blinded_pubkey, personal_data_hash },
			);
			VoterCount::<T>::put(new_voter_index);

			Ok(())
		}

		#[pallet::weight(0)]
		#[pallet::call_index(2)]
		pub fn update_candidate_info(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			name: String,
			pubkey: Vec<u8>,
		) -> DispatchResult {
			// make sure that it is signed by the candidate
			let sender = ensure_signed(origin)?;
			ensure!(sender == candidate, <Error<T>>::BadSender);

			// Update candidate info
			<Candidates<T>>::insert(candidate, Candidate { name, pubkey });

			Ok(())
		}

		#[pallet::weight(0)]
		#[pallet::call_index(3)]
		pub fn biased_signing(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			voter: u64,
			blinded_signature: BoundedVec<u8, T::SignatureLength>,
		) -> DispatchResult {
			// make sure that it is signed by the candidate
			let sender = ensure_signed(origin)?;
			ensure!(sender == candidate, <Error<T>>::BadSender);

			// Write to BlindedSignature
			<BlindedSignatures<T>>::insert(voter, candidate, blinded_signature);

			Ok(())
		}

		#[pallet::weight(0)]
		#[pallet::call_index(4)]
		pub fn vote(
			origin: OriginFor<T>,
			commitment: Vec<u8>,
			signature: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Votes can only be cast during the voting phase
			ensure!(Self::get_phase() == Some(ElectionPhase::Voting), <Error<T>>::InvalidPhase);

			// If the ballot already exists, update the vote
			if let Some(ballot) = <Ballots<T>>::get(sender.clone()) {
				// Update the ballot
				<Ballots<T>>::insert(sender, Ballot { commitment, signature, nonce: ballot.nonce + 1 });
			} else {
				// Add the ballot
				<Ballots<T>>::insert(sender, Ballot { commitment, signature, nonce: 1 });
			}

			Ok(())
		}

		#[pallet::weight(0)]
		#[pallet::call_index(6)]
		pub fn reveal_ballot_key(origin: OriginFor<T>, private_key: Vec<u8>) -> DispatchResult {
			// make sure that it is signed by the CA
			let sender = ensure_signed(origin)?;

			let ca = Self::ca();
			if let Some(ca) = ca {
				ensure!(sender == ca, <Error<T>>::SenderNotCA);
			} else {
				// if CA is not set, return error
				return Err(Error::<T>::InternalError.into())
			}

			// Ballot private key can only be revealed during the counting phase
			ensure!(Self::get_phase() == Some(ElectionPhase::Counting), <Error<T>>::InvalidPhase);

			let k = BallotKeys::<T>::get();
			if let Some(mut ballot_key) = k {
				// Update the ballot key
				ballot_key.private = private_key;
				<BallotKeys<T>>::set(Some(ballot_key));
			} else {
				return Err(Error::<T>::InternalError.into())
			}

			// TODO: Open the ballot

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_ca() -> Option<T::AccountId> {
			<CentralAuthority<T>>::get()
		}

		pub fn get_phase() -> Option<ElectionPhase> {
			<Phase<T>>::get()
		}

		pub fn get_voter(voter: u64) -> Option<Voter> {
			<Voters<T>>::get(voter)
		}

		pub fn get_candidate(candidate: T::AccountId) -> Option<Candidate> {
			<Candidates<T>>::get(candidate)
		}

		pub fn get_ballot(voter: T::AccountId) -> Option<Ballot> {
			<Ballots<T>>::get(voter)
		}

		pub fn get_ballot_key() -> Option<BallotKey> {
			BallotKeys::<T>::get()
		}
	}
}
