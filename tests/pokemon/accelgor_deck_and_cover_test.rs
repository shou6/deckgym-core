use deckgym::{
    actions::Action,
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::{attack_action, get_initialized_game},
};

fn played_card_with_base_hp(card_id: CardId, base_hp: u32, energy: Vec<EnergyType>) -> PlayedCard {
    PlayedCard::new(get_card_by_enum(card_id), 0, base_hp, energy, false, vec![])
}

/// Accelgor's "Deck and Cover": 50 damage, "Your opponent's Active Pokémon is now
/// Poisoned and Paralyzed. Shuffle this Pokémon and all attached cards into your
/// deck." The return is mandatory, unlike Eldegoss's Float Up.
#[test]
fn test_accelgor_deck_and_cover_poisons_paralyses_and_returns_itself() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B4014Accelgor)
                .with_energy(vec![EnergyType::Grass, EnergyType::Colorless]),
            PlayedCard::from_id(CardId::A1001Bulbasaur),
        ],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![],
        )],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    let deck_before = game.get_state_clone().decks[0].cards.len();

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B4014Accelgor, 0),
        is_stack: false,
    });

    // The return is mandatory, so it is the only action offered afterwards.
    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    assert_eq!(actor, 0, "the attacker resolves its own return");
    assert_eq!(
        choices.len(),
        1,
        "Deck and Cover is not optional, so there is nothing to decline: {choices:?}"
    );
    game.apply_action(&choices[0].clone());

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        250,
        "Deck and Cover should do 50 damage"
    );
    assert!(
        state.get_active(1).is_poisoned(),
        "the defender should be Poisoned"
    );
    assert!(
        state.get_active(1).is_paralyzed(),
        "the defender should be Paralyzed"
    );
    assert_eq!(
        state.decks[0].cards.len(),
        deck_before + 1,
        "Accelgor itself should be back in the deck"
    );
    assert!(
        state.in_play_pokemon[0][0]
            .as_ref()
            .is_none_or(|p| p.get_name() != "Accelgor"),
        "Accelgor should have left the Active Spot"
    );
}
