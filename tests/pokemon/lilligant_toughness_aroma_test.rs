use deckgym::{
    card_ids::CardId,
    models::{EnergyType, PlayedCard},
    test_support::get_initialized_game,
};

/// Lilligant's "Toughness Aroma": "Each of your [G] Pokémon gets +20 HP."
/// It is a board-wide passive, so it applies to every [G] Pokémon the owner has
/// in play - Lilligant itself included - and not to the opponent's.
#[test]
fn test_lilligant_toughness_aroma_adds_hp_to_own_grass_pokemon() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur).with_energy(vec![EnergyType::Grass]),
            PlayedCard::from_id(CardId::B1018Lilligant),
            PlayedCard::from_id(CardId::A1033Charmander),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    game.set_state(state);

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        90,
        "own [G] Bulbasaur should be 70 + 20"
    );
    assert_eq!(
        state.in_play_pokemon[0][1]
            .as_ref()
            .expect("Lilligant should be on the bench")
            .get_remaining_hp(),
        100,
        "Lilligant itself is [G], so it gets the bonus too (80 + 20)"
    );
    assert_eq!(
        state.in_play_pokemon[0][2]
            .as_ref()
            .expect("Charmander should be on the bench")
            .get_remaining_hp(),
        60,
        "Charmander is [R], so it gets nothing"
    );
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        70,
        "the opponent's [G] Bulbasaur should not benefit"
    );
}

/// Once Lilligant leaves play the bonus goes away.
#[test]
fn test_lilligant_toughness_aroma_stops_when_it_leaves_play() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur).with_energy(vec![EnergyType::Grass]),
            PlayedCard::from_id(CardId::B1018Lilligant),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    game.set_state(state);
    assert_eq!(game.get_state_clone().get_active(0).get_remaining_hp(), 90);

    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur).with_energy(vec![EnergyType::Grass])],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    game.set_state(state);
    assert_eq!(
        game.get_state_clone().get_active(0).get_remaining_hp(),
        70,
        "without Lilligant in play the bonus is gone"
    );
}
