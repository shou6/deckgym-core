use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    state::State,
    test_support::{attack_action, get_initialized_game},
};

fn played_card_with_base_hp(card_id: CardId, base_hp: u32, energy: Vec<EnergyType>) -> PlayedCard {
    PlayedCard::new(get_card_by_enum(card_id), 0, base_hp, energy, false, vec![])
}

fn can_retreat(state: &State) -> bool {
    let (_, actions) = state.generate_possible_actions();
    actions
        .iter()
        .any(|action| matches!(action.action, SimpleAction::Retreat(_)))
}

/// Salazzle's "Heated Poison": "Your opponent's Active Pokémon is now Poisoned and
/// Burned." Two conditions from one attack; Team Rocket's Houndoom's Toxfire Fang
/// shares the text.
#[test]
fn test_salazzle_heated_poison_applies_both_conditions() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A3036Salazzle)
            .with_energy(vec![EnergyType::Fire, EnergyType::Fire])],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![],
        )],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A3036Salazzle, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert!(
        state.get_active(1).is_poisoned(),
        "the defender should be Poisoned"
    );
    assert!(
        state.get_active(1).is_burned(),
        "the defender should be Burned"
    );
}

/// Heatran's ability: "If you have Arceus or Arceus ex in play, this Pokémon has
/// no Retreat Cost." Heatran's printed cost is 2, so the ability is the only way
/// it moves with no Energy.
#[test]
fn test_heatran_needs_arceus_for_free_retreat() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A2a013Heatran),
            PlayedCard::from_id(CardId::A1033Charmander),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);
    assert!(
        !can_retreat(&game.get_state_clone()),
        "without Arceus, Heatran pays its printed cost"
    );

    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A2a013Heatran),
            PlayedCard::from_id(CardId::A2a071ArceusEx),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    game.set_state(state);
    assert!(
        can_retreat(&game.get_state_clone()),
        "with Arceus ex in play Heatran retreats for free"
    );
}
