use crate::{mock::*, Error, Event, ElectionPhase};
use frame_support::{assert_noop, assert_ok};
use crate::ElectionPhase::Initialization;

#[test]
fn change_phase_works() {
	new_test_ext(1).execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Submit an extrinsic to change the phase
		assert_ok!(VotingSystem::change_phase(RuntimeOrigin::signed(1), Initialization));
		// Read pallet storage and assert an expected result.
		assert_eq!(VotingSystem::phase(), Some(Initialization));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::PhaseChanged { phase: Initialization, when: 1 }.into());
	});
}
