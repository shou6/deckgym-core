use deckgym::{
    actions::SimpleAction,
    card_ids::CardId,
    models::{EnergyType, PlayedCard},
    state::State,
    test_support::get_initialized_game,
};

/// Whether the player to act is offered a Retreat. Retreat only shows up once the
/// Active can pay its cost, so this is how an ability that removes the cost shows
/// through the public API.
fn can_retreat(state: &State) -> bool {
    let (_, actions) = state.generate_possible_actions();
    actions
        .iter()
        .any(|action| matches!(action.action, SimpleAction::Retreat(_)))
}

/// Wimpod's "Wimp Out": "During your first turn, this Pokémon has no Retreat
/// Cost." Its printed cost is 3 and it has no Energy, so a Retreat can only be
/// offered while the ability applies.
#[test]
fn test_wimpod_wimp_out_only_on_the_first_turn() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A3021Wimpod),
            PlayedCard::from_id(CardId::A1001Bulbasaur),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 1;
    game.set_state(state);
    assert!(
        can_retreat(&game.get_state_clone()),
        "Wimpod should have no Retreat Cost on the first turn"
    );

    let mut state = game.get_state_clone();
    state.turn_count = 5;
    game.set_state(state);
    assert!(
        !can_retreat(&game.get_state_clone()),
        "after the first turn Wimpod pays its printed cost of 3"
    );
}

/// Jumpluff's "Fluffy Flight": "Your Active Pokémon has no Retreat Cost." A
/// board-wide passive, so it works from the Bench and only for its owner.
#[test]
fn test_jumpluff_fluffy_flight_frees_your_active() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur),
            PlayedCard::from_id(CardId::A4015Jumpluff),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);
    assert!(
        can_retreat(&game.get_state_clone()),
        "Jumpluff on the bench should free the owner's Active"
    );

    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur),
            PlayedCard::from_id(CardId::A1033Charmander),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    game.set_state(state);
    assert!(
        !can_retreat(&game.get_state_clone()),
        "without Jumpluff the Active pays its printed cost"
    );
}

/// Energy on the Active is the ordinary way to retreat; this guards the helper
/// above from passing for the wrong reason.
#[test]
fn test_retreat_is_offered_when_energy_covers_the_cost() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur).with_energy(vec![EnergyType::Grass]),
            PlayedCard::from_id(CardId::A1033Charmander),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);
    assert!(
        can_retreat(&game.get_state_clone()),
        "Bulbasaur with 1 Energy can pay its cost of 1"
    );
}
