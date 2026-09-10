use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::PlayedCard,
    test_support::get_initialized_game,
};

/// Poltchageist's "Hospitality": "Once during your turn, when you put this Pokémon
/// from your hand onto your Bench, you may heal 20 damage from your Active [G]
/// Pokémon." Optional, triggered by being benched, and only heals a [G] Active.
#[test]
fn test_poltchageist_hospitality_heals_a_grass_active_when_benched() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur).with_remaining_hp(20)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    state.hands[0] = vec![get_card_by_enum(CardId::B4017Poltchageist)];
    game.set_state(state);

    let before = game.get_state_clone().get_active(0).get_remaining_hp();
    assert_eq!(before, 20, "Bulbasaur should start at 20 of its 70 HP");

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Place(get_card_by_enum(CardId::B4017Poltchageist), 1),
        is_stack: false,
    });

    // Being benched offers the heal; taking it should restore 20.
    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    assert_eq!(actor, 0, "the owner chooses");
    let use_ability = choices
        .iter()
        .find(|c| matches!(c.action, SimpleAction::UseAbility { .. }))
        .cloned()
        .expect(&format!("Hospitality should be offered: {choices:?}"));
    assert!(
        choices
            .iter()
            .any(|c| matches!(c.action, SimpleAction::Noop)),
        "it is optional, so declining must be possible"
    );

    game.apply_action(&use_ability);
    assert_eq!(
        game.get_state_clone().get_active(0).get_remaining_hp(),
        40,
        "Hospitality should heal 20"
    );
}

/// A non-[G] Active gets nothing, so the ability should not be offered.
#[test]
fn test_poltchageist_hospitality_skips_a_non_grass_active() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A1033Charmander).with_remaining_hp(20)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    state.hands[0] = vec![get_card_by_enum(CardId::B4017Poltchageist)];
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Place(get_card_by_enum(CardId::B4017Poltchageist), 1),
        is_stack: false,
    });

    let (_, choices) = game.get_state_clone().generate_possible_actions();
    assert!(
        !choices
            .iter()
            .any(|c| matches!(c.action, SimpleAction::UseAbility { .. })),
        "a [R] Active cannot be healed by Hospitality: {choices:?}"
    );
}
