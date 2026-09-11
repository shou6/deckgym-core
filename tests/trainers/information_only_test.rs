use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::get_initialized_game,
};

/// Play `trainer` for player 0 and return the state afterwards.
fn play_trainer(trainer: CardId) -> deckgym::State {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A1033Charmander).with_energy(vec![EnergyType::Fire])],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.hands[0] = vec![get_card_by_enum(trainer)];
    state.hands[1] = vec![get_card_by_enum(CardId::A1001Bulbasaur); 3];
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1001Bulbasaur); 6];
    state.decks[1].cards = vec![get_card_by_enum(CardId::A1001Bulbasaur); 6];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Play {
            trainer_card: get_card_by_enum(trainer).as_trainer(),
        },
        is_stack: false,
    });
    game.get_state_clone()
}

/// Cards that only let a player look at something. This engine has full information already, so
/// they are playable and change nothing: the deck keeps its order and its size, and no card
/// changes hands.
///
/// - Rotom Dex: "Look at the top card of your deck. Then, you may shuffle your deck."
/// - Looker: "Your opponent reveals all of the Supporter cards in their deck."
/// - Hand Scope: "Your opponent reveals their hand."
/// - Pokédex: "Look at the top 3 cards of your deck."
/// - Hiker / Morty: "look at that many cards from the top ... and put them back in any order."
///   The reordering is a real choice on paper; this engine does not model deck order choices, so
///   it is left as a no-op (documented in docs/deckgym-fork.md).
#[test]
fn test_information_only_trainers_change_nothing() {
    for trainer in [
        CardId::A3145RotomDEx,
        CardId::A3a068Looker,
        CardId::PA003HandScope,
        CardId::PA004PokedEx,
        CardId::A4161Hiker,
        CardId::A4a071Morty,
    ] {
        let state = play_trainer(trainer);
        assert_eq!(
            state.decks[0].cards.len(),
            6,
            "{trainer:?} leaves your deck alone",
        );
        assert_eq!(
            state.decks[1].cards.len(),
            6,
            "{trainer:?} leaves the opponent's deck alone",
        );
        assert!(
            state.hands[0].is_empty(),
            "{trainer:?} is the only card played: {:?}",
            state.hands[0],
        );
        assert_eq!(
            state.hands[1].len(),
            3,
            "{trainer:?} leaves the opponent's hand alone",
        );
    }
}
