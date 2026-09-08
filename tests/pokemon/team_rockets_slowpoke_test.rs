use deckgym::{
    actions::Action,
    card_ids::CardId,
    database::get_card_by_enum,
    models::{Card, EnergyType, PlayedCard},
    test_support::{attack_action, get_test_game_with_board},
};

/// Team Rocket's Slowpoke's Scavenge: "Put a random Item card from your discard pile into your
/// hand." The attack deals no damage; its value is recovering an Item.
#[test]
fn test_scavenge_puts_an_item_from_the_discard_pile_into_the_hand() {
    let mut game = get_test_game_with_board(
        vec![PlayedCard::from_id(CardId::B4a025TeamRocketsSlowpoke)
            .with_energy(vec![EnergyType::Psychic])],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );

    // Put one Item (Poké Ball) and one non-Item (Professor's Research) in the discard pile.
    let poke_ball = get_card_by_enum(CardId::PA005PokeBall);
    let research = get_card_by_enum(CardId::PA007ProfessorsResearch);
    let mut state = game.get_state_clone();
    state.discard_piles[0] = vec![poke_ball.clone(), research.clone()];
    state.hands[0].clear();
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B4a025TeamRocketsSlowpoke, 0),
        is_stack: false,
    });

    let after = game.get_state_clone();
    assert!(
        after.hands[0].contains(&poke_ball),
        "the Item should be recovered into the hand, hand was {:?}",
        after.hands[0]
    );
    assert!(
        !after.discard_piles[0].contains(&poke_ball),
        "the recovered Item should leave the discard pile"
    );
    assert!(
        after.discard_piles[0].contains(&research),
        "a Supporter is not an Item and should stay in the discard pile"
    );
}

/// With no Item in the discard pile the attack simply does nothing.
#[test]
fn test_scavenge_without_any_item_leaves_the_hand_unchanged() {
    let mut game = get_test_game_with_board(
        vec![PlayedCard::from_id(CardId::B4a025TeamRocketsSlowpoke)
            .with_energy(vec![EnergyType::Psychic])],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );

    let research = get_card_by_enum(CardId::PA007ProfessorsResearch);
    let mut state = game.get_state_clone();
    state.discard_piles[0] = vec![research.clone()];
    state.hands[0].clear();
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B4a025TeamRocketsSlowpoke, 0),
        is_stack: false,
    });

    let after = game.get_state_clone();
    let items: Vec<&Card> = after.hands[0]
        .iter()
        .filter(|card| matches!(card, Card::Trainer(t) if t.trainer_card_type == deckgym::models::TrainerType::Item))
        .collect();
    assert!(
        items.is_empty(),
        "nothing should be recovered, hand was {:?}",
        after.hands[0]
    );
}
