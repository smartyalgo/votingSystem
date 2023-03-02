use crate::{
	mock::*,
	Ballot, Candidate,
	ElectionPhase::{Initialization, Registration},
	Error, Event, Voter,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn e2e() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		let ca = root_key;
		System::set_block_number(1);
		// Initialization phase
		assert_eq!(VotingSystem::phase(), Some(Initialization));
		assert_eq!(VotingSystem::get_ca(), Some(1));
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));

		// Registration phase
		let voter = 1;
		let blinded_pubkey = vec![1, 2, 3];
		let signed_blinded_pubkey = vec![4, 5, 6];
		let is_eligible = true;
		let personal_data_hash = vec![7, 8, 9];
		assert_ok!(VotingSystem::add_voter(
			RuntimeOrigin::signed(ca),
			blinded_pubkey.clone(),
			signed_blinded_pubkey.clone(),
			personal_data_hash.clone(),
			is_eligible
		));
		assert_eq!(
			VotingSystem::voters(1),
			Some(Voter { blinded_pubkey, signed_blinded_pubkey, is_eligible, personal_data_hash })
		);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));

		// TODO: Biased Signer phase
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));

		// Voting phase
		let commitment = vec![1, 2, 3];
		let signature = vec![4, 5, 6];

		assert_ok!(VotingSystem::vote(
			RuntimeOrigin::signed(voter),
			commitment.clone(),
			signature.clone()
		));
		assert_eq!(
			VotingSystem::get_ballot(voter),
			Some(Ballot { commitment, signature, nonce: 1 })
		);

		let new_commitment = vec![1, 2, 3, 4];
		let new_signature = vec![4, 5, 6, 7];
		assert_ok!(VotingSystem::change_vote(
			RuntimeOrigin::signed(voter),
			new_commitment.clone(),
			new_signature.clone()
		));
		assert_eq!(
			VotingSystem::get_ballot(voter),
			Some(Ballot { commitment: new_commitment, signature: new_signature, nonce: 2 })
		);

		// TODO: Counting phase
	})
}

#[test]
fn change_phase_works() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let ca = root_key;

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));

		// then
		assert_eq!(VotingSystem::phase(), Some(Registration));
		System::assert_last_event(Event::PhaseChanged { phase: Registration, when: 1 }.into());
	});
}

#[test]
fn change_phase_errors_when_not_ca() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let nonce: u64 = 2;

		// when
		System::set_block_number(1);

		// then
		assert_noop!(
			VotingSystem::change_phase(RuntimeOrigin::signed(nonce)),
			Error::<Test>::SenderNotCA
		);
	});
}

#[test]
fn can_add_voter() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let ca = root_key;
		let voter = 1;
		let blinded_pubkey = vec![1, 2, 3];
		let signed_blinded_pubkey = vec![4, 5, 6];
		let is_eligible = true;
		let personal_data_hash = vec![7, 8, 9];

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::add_voter(
			RuntimeOrigin::signed(ca),
			blinded_pubkey.clone(),
			signed_blinded_pubkey.clone(),
			personal_data_hash.clone(),
			is_eligible
		));

		// then
		assert_eq!(
			VotingSystem::voters(voter),
			Some(Voter { blinded_pubkey, signed_blinded_pubkey, is_eligible, personal_data_hash })
		);
	})
}

#[test]
fn can_update_candidate() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let candidate = 2;
		let name = "candidate 1";
		let pubkey: Vec<u8> = vec![1, 2, 3];

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::update_candidate_info(
			RuntimeOrigin::signed(candidate),
			candidate,
			name.to_string(),
			pubkey.clone()
		));

		// then
		assert_eq!(
			VotingSystem::get_candidate(candidate),
			Some(Candidate { name: name.to_string(), pubkey })
		);
	})
}

#[test]
fn can_vote() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let voter = 5;
		let ca = 1;
		let blinded_pubkey = vec![1, 2, 3];
		let signed_blinded_pubkey = vec![4, 5, 6];
		let is_eligible = true;
		let personal_data_hash = vec![7, 8, 9];
		let commitment = vec![1, 2, 3];
		let signature = vec![4, 5, 6];

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::add_voter(
			RuntimeOrigin::signed(ca),
			blinded_pubkey,
			signed_blinded_pubkey,
			personal_data_hash,
			is_eligible,
		));
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));

		// then vote
		assert_ok!(VotingSystem::vote(
			RuntimeOrigin::signed(voter),
			commitment.clone(),
			signature.clone()
		));
		assert_eq!(
			VotingSystem::get_ballot(voter),
			Some(Ballot { commitment, signature, nonce: 1 })
		);
	})
}

#[test]
pub fn can_change_vote() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let voter = 5;
		let ca = 1;
		let blinded_pubkey = vec![1, 2, 3];
		let signed_blinded_pubkey = vec![4, 5, 6];
		let is_eligible = true;
		let commitment = vec![1, 2, 3];
		let signature = vec![4, 5, 6];
		let personal_data_hash = vec![7, 8, 9];

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::add_voter(
			RuntimeOrigin::signed(ca),
			blinded_pubkey,
			signed_blinded_pubkey,
			personal_data_hash,
			is_eligible
		));
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::vote(
			RuntimeOrigin::signed(voter),
			commitment.clone(),
			signature.clone()
		));
		assert_eq!(
			VotingSystem::get_ballot(voter),
			Some(Ballot { commitment, signature, nonce: 1 })
		);

		// then change vote
		let new_commitment = vec![1, 2, 3, 4];
		let new_signature = vec![4, 5, 6, 7];
		assert_ok!(VotingSystem::change_vote(
			RuntimeOrigin::signed(voter),
			new_commitment.clone(),
			new_signature.clone()
		));
		assert_eq!(
			VotingSystem::get_ballot(voter),
			Some(Ballot { commitment: new_commitment, signature: new_signature, nonce: 2 })
		);
	})
}

#[test]
// TODO: Incomplete test
#[ignore]
fn can_biased_signing() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let candidate = 2;
		let voter: u64 = 15;
		// let pubkey: Vec<u8> = vec![1, 2, 3];
		// let blinded_signature: BoundedVec<u8, Get<u32>> = vec![1, 2, 3];

		// when
		System::set_block_number(1);
		// assert_ok!(VotingSystem::biased_signing(
		// 	RuntimeOrigin::signed(candidate),
		// 	candidate,
		// 	voter,
		// 	blinded_signature
		// ));

		// then
		// assert_eq!(
		// 	VotingSystem::get_candidate(candidate),
		// 	Some(Candidate { name: name.to_string(), pubkey })
		// );
	})
}
