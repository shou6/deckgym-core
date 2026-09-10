use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    models::PlayedCard,
    test_support::get_initialized_game,
};

fn ability_action(state: &deckgym::State, in_play_idx: usize) -> Option<Action> {
    let (_, actions) = state.generate_possible_actions();
    actions
        .iter()
        .find(|a| matches!(a.action, SimpleAction::UseAbility { in_play_idx: idx } if idx == in_play_idx))
        .cloned()
}

/// Rillaboom's "Captivating Rhythm": "Once during your turn, you may flip a coin.
/// If heads, switch in 1 of your opponent's Benched Pokémon to the Active Spot."
/// Unlike Umbreon's Dark Chase it works from the Bench and does not care whether
/// the target is damaged.
#[test]
fn test_rillaboom_captivating_rhythm_is_offered_from_the_bench() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur),
            PlayedCard::from_id(CardId::B1027Rillaboom),
        ],
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur),
            PlayedCard::from_id(CardId::A1033Charmander),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    assert!(
        ability_action(&game.get_state_clone(), 1).is_some(),
        "Captivating Rhythm should be usable from the bench"
    );
}

/// With no Benched Pokemon on the opponent's side there is nothing to switch in,
/// so the ability should not be offered.
#[test]
fn test_rillaboom_captivating_rhythm_needs_an_opponent_bench() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur),
            PlayedCard::from_id(CardId::B1027Rillaboom),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    assert!(
        ability_action(&game.get_state_clone(), 1).is_none(),
        "with an empty opponent bench there is nothing to switch in"
    );
}

/// Using it flips a coin; on heads the opponent's Active changes. Seeds differ, so
/// drive both sides by checking that the ability resolves and is then spent.
#[test]
fn test_rillaboom_captivating_rhythm_is_once_per_turn() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur),
            PlayedCard::from_id(CardId::B1027Rillaboom),
        ],
        vec![
            PlayedCard::from_id(CardId::A1001Bulbasaur),
            PlayedCard::from_id(CardId::A1033Charmander),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    let action = ability_action(&game.get_state_clone(), 1).expect("ability should be offered");
    game.apply_action(&action);
    game.play_until_stable();

    assert!(
        ability_action(&game.get_state_clone(), 1).is_none(),
        "the ability is once per turn, so it should not be offered again"
    );
}
