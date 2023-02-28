use crate::{mock::*, ElectionPhase::None, ElectionPhase::Initialization, Error, Event};
use frame_support::{assert_ok, assert_err_ignore_postinfo, assert_noop};

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
		let nonca: u64 = 2;
		let phase = None;

		// when
		System::set_block_number(1);
		// VotingSystem::change_phase(RuntimeOrigin::signed(nonca), phase.clone())

		assert_noop!(VotingSystem::change_phase(RuntimeOrigin::signed(nonca), phase.clone()), Error::<Test>::SenderNotCA);
	});
}
