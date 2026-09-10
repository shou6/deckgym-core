use deckgym::{
    actions::SimpleAction, card_ids::CardId, database::get_card_by_enum, models::PlayedCard,
    state::State, test_support::get_initialized_game,
};

/// Whether the player to act can use an attack right now.
fn can_attack(state: &State) -> bool {
    let (_, actions) = state.generate_possible_actions();
    actions
        .iter()
        .any(|action| matches!(action.action, SimpleAction::Attack(_)))
}

/// Cherubi's "En-fruits-iastic": "If this Pokémon has a Pokémon Tool attached,
/// attacks used by this Pokémon cost 1 less [G] Energy." Sweets Relay costs one
/// [G], so with a Tool attached it can attack with no Energy at all.
#[test]
fn test_cherubi_enfruitsiastic_needs_a_tool() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A4023Cherubi)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);
    assert!(
        !can_attack(&game.get_state_clone()),
        "without a Tool, Cherubi still needs its [G] Energy"
    );

    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A4023Cherubi)
            .with_tool(get_card_by_enum(CardId::A3147LeafCape))],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    game.set_state(state);
    assert!(
        can_attack(&game.get_state_clone()),
        "with a Tool attached the [G] cost drops to zero"
    );
}
