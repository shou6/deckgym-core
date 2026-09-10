use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::{attack_action, get_initialized_game},
};

fn played_card_with_base_hp(card_id: CardId, base_hp: u32, energy: Vec<EnergyType>) -> PlayedCard {
    PlayedCard::new(get_card_by_enum(card_id), 0, base_hp, energy, false, vec![])
}

/// Roserade's "Poison Ring": 50 damage, "Your opponent's Active Pokémon is now
/// Poisoned. During your opponent's next turn, that Pokémon can't retreat."
/// Both halves have to land.
#[test]
fn test_roserade_poison_ring_poisons_and_locks_the_defender() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2005Roserade)
            .with_energy(vec![EnergyType::Grass, EnergyType::Colorless])],
        vec![
            played_card_with_base_hp(
                CardId::A1001Bulbasaur,
                300,
                vec![EnergyType::Grass, EnergyType::Grass],
            ),
            PlayedCard::from_id(CardId::A1033Charmander),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2005Roserade, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        250,
        "Poison Ring should do 50 damage"
    );
    assert!(
        state.get_active(1).is_poisoned(),
        "the defender should be Poisoned"
    );

    // Hand the turn over; the defender has Energy to spare but must not be able
    // to retreat.
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });
    let (_, actions) = game.get_state_clone().generate_possible_actions();
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Retreat(_))),
        "the Poisoned defender must not be able to retreat: {actions:?}"
    );
}
