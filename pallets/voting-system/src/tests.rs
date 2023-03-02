use crate::{mock::*, Candidate, ElectionPhase::Registration, Error, Event, Voter, Ballot};
use frame_support::{assert_noop, assert_ok};

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
		let blinded_pubkey = vec![1, 2, 3];
		let signed_blinded_pubkey = vec![4, 5, 6];
		let is_eligible = true;

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::add_voter(
			RuntimeOrigin::signed(ca),
			blinded_pubkey.clone(),
			signed_blinded_pubkey.clone(),
			is_eligible
		));

		// then
		assert_eq!(
			VotingSystem::voters(1),
			Some(Voter { id: 1, blinded_pubkey, signed_blinded_pubkey, is_eligible })
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

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::update_candidate_info(
			RuntimeOrigin::signed(candidate),
			candidate,
			name.to_string()
		));

		// then
		assert_eq!(VotingSystem::get_candidate(candidate), Some(Candidate { name: name.to_string() }));
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
		let commitment = vec![1, 2, 3];
		let signature = vec![4, 5, 6];

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::add_voter(
			RuntimeOrigin::signed(ca),
			blinded_pubkey.clone(),
			signed_blinded_pubkey.clone(),
			is_eligible
		));
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));

		// then vote
		assert_ok!(VotingSystem::vote(RuntimeOrigin::signed(voter), voter, commitment.clone(), signature.clone()));
		assert_eq!(VotingSystem::get_ballot(voter), Some(Ballot { voter_id: voter, commitment, signature }));
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

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::add_voter(
			RuntimeOrigin::signed(ca),
			blinded_pubkey.clone(),
			signed_blinded_pubkey.clone(),
			is_eligible
		));
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca)));
		assert_ok!(VotingSystem::vote(RuntimeOrigin::signed(voter), voter, commitment.clone(), signature.clone()));
		assert_eq!(VotingSystem::get_ballot(voter), Some(Ballot { voter_id: voter, commitment, signature }));

		// then change vote
		let new_commitment = vec![1, 2, 3, 4];
		let new_signature = vec![4, 5, 6, 7];
		assert_ok!(VotingSystem::change_vote(RuntimeOrigin::signed(voter), voter, new_commitment.clone(), new_signature.clone()));
		assert_eq!(
			VotingSystem::get_ballot(voter),
			Some(Ballot { voter_id: voter, commitment: new_commitment, signature: new_signature })
		);
	})
}
