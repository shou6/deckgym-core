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

/// Eldegoss's "Float Up": 40 damage, then "You may shuffle this Pokémon and all
/// attached cards into your deck." Optional, so the player is offered the choice
/// and can decline.
#[test]
fn test_eldegoss_float_up_offers_the_choice_and_can_return_itself() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B2016Eldegoss)
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
    game.set_state(state);

    let deck_before = game.get_state_clone().decks[0].cards.len();

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2016Eldegoss, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        260,
        "Float Up should do 40 damage (300 - 40 = 260)"
    );

    let (actor, choices) = state.generate_possible_actions();
    assert_eq!(actor, 0, "the attacker chooses");
    assert!(
        choices
            .iter()
            .any(|c| matches!(c.action, SimpleAction::Noop)),
        "declining must be possible: {choices:?}"
    );
    let shuffle = choices
        .iter()
        .find(|c| !matches!(c.action, SimpleAction::Noop))
        .expect("there should be a way to shuffle Eldegoss back")
        .clone();

    game.apply_action(&shuffle);

    let state = game.get_state_clone();
    assert_eq!(
        state.decks[0].cards.len(),
        deck_before + 1,
        "Eldegoss itself should be back in the deck"
    );
    assert!(
        state.in_play_pokemon[0][0]
            .as_ref()
            .is_none_or(|p| p.get_name() != "Eldegoss"),
        "Eldegoss should have left the Active Spot"
    );
}

/// Declining leaves the board untouched.
#[test]
fn test_eldegoss_float_up_can_be_declined() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B2016Eldegoss)
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
    game.set_state(state);

    let deck_before = game.get_state_clone().decks[0].cards.len();

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2016Eldegoss, 0),
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Noop,
        is_stack: true,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.decks[0].cards.len(),
        deck_before,
        "declining should leave the deck alone"
    );
    assert_eq!(
        state.get_active(0).get_name(),
        "Eldegoss",
        "Eldegoss should still be Active"
    );
}
