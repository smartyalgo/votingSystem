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
	use frame_system::{pallet_prelude::*};
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
		pub signature: Vec<u8>, // TODO: There needs to be one for each candidate
		pub nonce: u64,
	}
	/// Todo: determine maximum length of struct storage
	impl MaxEncodedLen for Ballot {
		fn max_encoded_len() -> usize {
			usize::MAX - 1
		}
	}
	
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct BlindSignature {
		// Candidate Lookup key
		// TODO: How do we store an account ID here, whats the type?
		// pub acconut: T::AccountId,
		pub signature: Vec<u8>,
		pub msg_randomizer: [u8; 32],
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
	#[pallet::getter(fn candidates_count)]
	pub type CandidatesCount<T: Config> = StorageValue<_, u64, OptionQuery>;

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
		/// Voter ID is invalid
		VoterDoesNotExist,
		/// Invalid phase change
		InvalidPhaseChange,
		/// Invalid phase
		InvalidPhase,
		/// Bad Sender
		BadSender,
		/// Missing count of candidates
		MissingCandidateCount,
		/// Ballot already exists
		BallotAlreadyExists,
		/// Duplicate or missing blind signatures
		InvalidBlindSignatures,
		/// Ballot does not exist
		BallotNotFound,
		/// RSA Key Storage not found
		RSAStorageNotFound,
		/// Invalid RSA Key in storage
		RSAError,
		/// Bad Signature from RSA Key
		RSAInvalidSignature,
		/// Invalid public key
		InvalidPublicKey
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
			CandidatesCount::<T>::put(self.candidates.len() as u64);
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

			// TODO: Pull this out into it's own function for testability
			// Additional phase-specific logic check if current phase can be ended

			// TODO: Change this logic, in biased_sign function, per candidate
			// keeps track of how many endorsement they have made. Only
			//  proceed if for all candidates the count == voter count
			let current_phase = Self::phase();
			match current_phase {
				Some(ElectionPhase::BiasedSigner) => {
					// Check if all the voters has received all blinded signatures from all candidates
					// For each voter, check if blinded signature array == candidate count
					let mut voter_index = 1;
					while Some(voter_index) <= Self::voter_count() {
						// Get BlindedSignature(voter_id, candidate)
						let mut blinded_signature_count: u64 = 0;
						BlindedSignatures::<T>::iter_prefix(voter_index).for_each(
							|(_candidate, _)| {
								blinded_signature_count += 1;
							},
						);
						if Some(blinded_signature_count) != Self::candidates_count() {
							return Err(Error::<T>::InvalidPhaseChange.into());
						}
						voter_index += 1;
					}
				},
				_ => {},
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

			// Fetch the voters blinded key to verify the signature
			let voter_data;
			
			match Self::get_voter(voter) {
				Some(data) => voter_data = data,
				None => return Err(Error::<T>::VoterDoesNotExist.into()),
			}

			// Fetch the candidates public key
			// let rsa_public: blind_rsa_signatures::reexports::rsa::RsaPublicKey;
			let rsa_public : blind_rsa_signatures::PublicKey;
			if let Some(candidate_struct) = Self::get_candidate(candidate.clone()) {
				// let res = blind_rsa_signatures::reexports::rsa::RsaPublicKey::from_public_key_der(candidate_struct.pubkey.as_slice());
				let res = blind_rsa_signatures::PublicKey::from_der(candidate_struct.pubkey.as_slice());
				if let Ok(key) = res {
					rsa_public = key;
				} else {
					return Err(Error::<T>::InvalidPublicKey.into());
				}
			} else {
				return Err(Error::<T>::RSAStorageNotFound.into());
			}

			// Format the signature correctly
			let signature = blind_rsa_signatures::Signature::new(blinded_signature.to_vec());

			// Set the verification options
			let options = blind_rsa_signatures::Options::default();
			
			// Verify the signatures match the candidates public key
			let verification = rsa_public.verify(&signature, None, voter_data.blinded_pubkey, &options);
				
			// If Verification fails we need to kill the transaction
			if verification.is_err() {
				return Err(Error::<T>::RSAInvalidSignature.into());
			}
		
			// Write to BlindedSignature
				<BlindedSignatures<T>>::insert(voter, candidate, blinded_signature);

			Ok(())
		}

		#[pallet::weight(0)]
		#[pallet::call_index(4)]
		pub fn vote(
			origin: OriginFor<T>,
			commitment: Vec<u8>,
			mut signature_set: Vec<(T::AccountId, BlindSignature)>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Fetch the voters public key from their AccountID
			let voter_public_key: Vec<u8> = sender.encode();

			// Votes can only be cast during the voting phase
			ensure!(Self::get_phase() == Some(ElectionPhase::Voting), <Error<T>>::InvalidPhase);

			// Get the total count of candidates
			let candidate_count: u64;
			if let Some(count) = CandidatesCount::<T>::get()  {
				candidate_count = count;
			} else {
				return Err(Error::<T>::MissingCandidateCount.into());
			}

			// Check if the number of signatures does not match the number of expected candidates signatures
			if candidate_count as usize != signature_set.len() {
				return Err(Error::<T>::InvalidBlindSignatures.into());
			}

			// Sort the list of signatures so we can later verify that no two signatures match to 
			// prevent a user submitting multiple of the same signature while only using O(N) time
			signature_set.sort_by(|a,b| b.0.cmp(&a.0));

			// Verify that the ballot is valid by checking for all candidates signatures
			let mut last_id: Option<T::AccountId> = None; 
			for signature in signature_set {
				let candidate_id = signature.0;
				let blind_signature = signature.1;
				// If the last candidate id is equal to or greater then the last there are duplicate entries
				if let Some(id) = last_id {
					if  id >= candidate_id {
						return Err(Error::<T>::InvalidBlindSignatures.into());
					}
				}
				// Update the last id for the next loops check
				last_id = Some(candidate_id.clone());

				// Verify the actual signatures to make sure they came from a candidate
				// Start by trying to fetch the candidates public key
				let rsa_public : blind_rsa_signatures::PublicKey;
				if let Some(candidate_struct) = Self::get_candidate(candidate_id.clone()) {
					let res = blind_rsa_signatures::PublicKey::from_der(candidate_struct.pubkey.as_slice());
					if let Ok(key) = res {
						rsa_public = key;
					} else {
						return Err(Error::<T>::InvalidBlindSignatures.into());
					}
				} else {
					return Err(Error::<T>::InvalidBlindSignatures.into());
				}

				// Format the signature correctly
				let signature = blind_rsa_signatures::Signature::new(blind_signature.signature.to_vec());

				// Set the verification options
				let options = blind_rsa_signatures::Options::default();

				// Decode the Message Randomizer Correctly
				let msg_randomizer = Some(blind_rsa_signatures::MessageRandomizer::from(blind_signature.msg_randomizer));
				
				// Verify the signatures match the candidates public key
				let verification = rsa_public.verify(&signature, msg_randomizer, voter_public_key.clone(), &options);
					
				// If Verification fails we need to kill the transaction
				if verification.is_err() {
					return Err(Error::<T>::InvalidBlindSignatures.into());
				}
				
			}

			// If the ballot already exists, update the vote
			if let Some(ballot) = <Ballots<T>>::get(sender.clone()) {
				// Update the ballot
				<Ballots<T>>::insert(
					sender,
					Ballot { commitment, signature:Vec::new(), nonce: ballot.nonce + 1 },
				);
			} else {
				// Add the ballot
				<Ballots<T>>::insert(sender, Ballot { commitment, signature: Vec::new(), nonce: 1 });
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
				return Err(Error::<T>::InternalError.into());
			}

			// Ballot private key can only be revealed during the counting phase
			ensure!(Self::get_phase() == Some(ElectionPhase::Counting), <Error<T>>::InvalidPhase);

			let k = BallotKeys::<T>::get();
			if let Some(mut ballot_key) = k {
				// Update the ballot key
				ballot_key.private = private_key;
				<BallotKeys<T>>::set(Some(ballot_key));
			} else {
				return Err(Error::<T>::InternalError.into());
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