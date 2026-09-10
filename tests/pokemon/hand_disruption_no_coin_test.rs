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

/// Tsareena's "Kick Down": 50 damage, then "Your opponent reveals a random card
/// from their hand and shuffles it into their deck." No coin flip, so the
/// opponent always loses a card from hand and gains one in their deck.
#[test]
fn test_tsareena_kick_down_shuffles_a_hand_card_into_the_deck() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![PlayedCard::from_id(CardId::A3b005Tsareena).with_energy(vec![EnergyType::Grass])],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![EnergyType::Grass],
        )],
    );
    state.current_player = 0;
    game.set_state(state);

    let before = game.get_state_clone();
    let hand_before = before.hands[1].len();
    let deck_before = before.decks[1].cards.len();
    assert!(hand_before > 0, "opponent should hold cards to lose");

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A3b005Tsareena, 0),
        is_stack: false,
    });

    let after = game.get_state_clone();
    assert_eq!(
        after.get_active(1).get_remaining_hp(),
        250,
        "Bulbasaur should take 50 damage from Kick Down"
    );
    assert_eq!(
        after.hands[1].len(),
        hand_before - 1,
        "one card should leave the opponent's hand"
    );
    assert_eq!(
        after.decks[1].cards.len(),
        deck_before + 1,
        "that card should go back into the opponent's deck"
    );
}

/// Shiftry's "Nipping Cyclone": 70 damage, then "Discard a random card from your
/// opponent's hand." No coin flip, and the card goes to the discard pile.
#[test]
fn test_shiftry_nipping_cyclone_discards_a_hand_card() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![PlayedCard::from_id(CardId::B1010Shiftry)
            .with_energy(vec![EnergyType::Grass, EnergyType::Grass])],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![EnergyType::Grass],
        )],
    );
    state.current_player = 0;
    game.set_state(state);

    let before = game.get_state_clone();
    let hand_before = before.hands[1].len();
    let discard_before = before.discard_piles[1].len();
    assert!(hand_before > 0, "opponent should hold cards to lose");

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B1010Shiftry, 0),
        is_stack: false,
    });

    let after = game.get_state_clone();
    assert_eq!(
        after.get_active(1).get_remaining_hp(),
        230,
        "Bulbasaur should take 70 damage from Nipping Cyclone"
    );
    assert_eq!(
        after.hands[1].len(),
        hand_before - 1,
        "one card should leave the opponent's hand"
    );
    assert_eq!(
        after.discard_piles[1].len(),
        discard_before + 1,
        "that card should go to the opponent's discard pile"
    );
}
