use crate::{mock::*, ElectionPhase::None, ElectionPhase::Initialization, Error, Event, Candidate};
use frame_support::{assert_ok, assert_noop};
use crate::{ElectionPhase::Registration, Voter};

#[test]
fn change_phase_works() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let ca = root_key;
		let phase = Initialization;

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca), phase.clone()));

		// then
		assert_eq!(VotingSystem::phase(), Some(phase.clone()));
		System::assert_last_event(Event::PhaseChanged { phase, when: 1 }.into());
	});
}

#[test]
fn change_phase_errors_when_not_ca() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let nonce: u64 = 2;
		let phase = None;

		// when
		System::set_block_number(1);
		// VotingSystem::change_phase(RuntimeOrigin::signed(nonce), phase.clone())

		assert_noop!(VotingSystem::change_phase(RuntimeOrigin::signed(nonce), phase.clone()), Error::<Test>::SenderNotCA);
	});
}

#[test]
fn can_add_voter() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let ca = root_key;
		let voter = 2;
		let blinded_pubkey = vec![1, 2, 3];
		let signed_blinded_pubkey = vec![4, 5, 6];
		let is_eligible = true;

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(ca), Registration));
		assert_ok!(VotingSystem::add_voter(RuntimeOrigin::signed(ca), voter, blinded_pubkey.clone(), signed_blinded_pubkey.clone(), is_eligible));

		// then
		assert_eq!(VotingSystem::voters(voter), Some(Voter {
			blinded_pubkey,
			signed_blinded_pubkey,
			is_eligible,
		}));
	})
}

#[test]
fn can_update_candidate() {
	let root_key = 1;
	new_test_ext(root_key).execute_with(|| {
		// with
		let candidate = 2;
		let name = vec![1, 2, 3];

		// when
		System::set_block_number(1);
		assert_ok!(VotingSystem::update_candidate_info(RuntimeOrigin::signed(candidate), candidate, name.clone()));

		// then
		assert_eq!(VotingSystem::get_candidate(candidate), Some(Candidate {name}));
	})
}
