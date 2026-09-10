use deckgym::{
    actions::Action,
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::{attack_action, get_initialized_game},
};

/// Celebi's "Temporal Leaves": 40 damage, "If your opponent's Active Pokémon is an
/// evolved Pokémon, devolve it by putting the highest Stage Evolution card on it
/// into your opponent's hand." Damage, energy and tools stay on what is left.
#[test]
fn test_celebi_temporal_leaves_devolves_the_defender() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    // Ivysaur standing on a Bulbasaur, with Energy attached.
    let mut evolved = PlayedCard::from_id(CardId::A1002Ivysaur)
        .with_energy(vec![EnergyType::Grass, EnergyType::Grass]);
    evolved.cards_behind = vec![get_card_by_enum(CardId::A1001Bulbasaur)];

    state.set_board(
        vec![PlayedCard::from_id(CardId::A4a006Celebi).with_energy(vec![EnergyType::Grass])],
        vec![evolved],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    let hand_before = game.get_state_clone().hands[1].len();

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A4a006Celebi, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    let defender = state.get_active(1);
    assert_eq!(
        defender.get_name(),
        "Bulbasaur",
        "Ivysaur should have been peeled off, leaving Bulbasaur"
    );
    assert_eq!(
        defender.attached_energy.len(),
        2,
        "Energy stays on the Pokemon that remains"
    );
    assert_eq!(
        state.hands[1].len(),
        hand_before + 1,
        "Ivysaur should be back in its owner's hand"
    );
    // Bulbasaur is 70 HP and took 40.
    assert_eq!(
        defender.get_remaining_hp(),
        30,
        "damage carries over to what is left"
    );
}

/// A Basic defender has nothing to peel off, so it just takes the damage.
#[test]
fn test_celebi_temporal_leaves_leaves_a_basic_alone() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A4a006Celebi).with_energy(vec![EnergyType::Grass])],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    let hand_before = game.get_state_clone().hands[1].len();

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A4a006Celebi, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_name(), "Bulbasaur");
    assert_eq!(state.get_active(1).get_remaining_hp(), 30, "70 - 40 = 30");
    assert_eq!(
        state.hands[1].len(),
        hand_before,
        "nothing goes back to hand"
    );
}
