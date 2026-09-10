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

fn setup<'a>(attacker: CardId, energy: Vec<EnergyType>) -> deckgym::Game<'a> {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![played_card_with_base_hp(attacker, 300, energy)],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![],
        )],
    );
    state.current_player = 0;
    game.set_state(state);
    game
}

fn attack_and_read_defender_hp(game: &mut deckgym::Game<'_>, attacker: CardId) -> u32 {
    game.apply_action(&Action {
        actor: 0,
        action: attack_action(attacker, 0),
        is_stack: false,
    });
    game.get_state_clone().get_active(1).get_remaining_hp()
}

/// Buzzwole's "Ground Beat": 40 damage, "+40 if your opponent has gotten exactly
/// 1 points". Exactly 1 - neither 0 nor 2 counts.
#[test]
fn test_buzzwole_ground_beat_needs_exactly_one_opponent_point() {
    for (points, expected_hp) in [(0u8, 260u32), (1, 220), (2, 260)] {
        let mut game = setup(
            CardId::B2014Buzzwole,
            vec![EnergyType::Fighting, EnergyType::Colorless],
        );
        let mut state = game.get_state_clone();
        state.points[1] = points;
        game.set_state(state);

        assert_eq!(
            attack_and_read_defender_hp(&mut game, CardId::B2014Buzzwole),
            expected_hp,
            "opponent at {points} points should leave Bulbasaur at {expected_hp}"
        );
    }
}

/// Pheromosa's "Prelude": 30 damage, "+60 if you haven't gotten any points".
#[test]
fn test_pheromosa_prelude_needs_zero_own_points() {
    for (points, expected_hp) in [(0u8, 210u32), (1, 270)] {
        let mut game = setup(
            CardId::B4016Pheromosa,
            vec![EnergyType::Grass, EnergyType::Colorless],
        );
        let mut state = game.get_state_clone();
        state.points[0] = points;
        game.set_state(state);

        assert_eq!(
            attack_and_read_defender_hp(&mut game, CardId::B4016Pheromosa),
            expected_hp,
            "attacker at {points} points should leave Bulbasaur at {expected_hp}"
        );
    }
}

/// Illumise's "Ire-Fly": 30 damage, "+60 if Volbeat is in your discard pile".
#[test]
fn test_illumise_ire_fly_needs_volbeat_in_discard() {
    let mut game = setup(CardId::B4a002Illumise, vec![EnergyType::Grass]);
    assert_eq!(
        attack_and_read_defender_hp(&mut game, CardId::B4a002Illumise),
        270,
        "without Volbeat in the discard pile it should do a flat 30"
    );

    let mut game = setup(CardId::B4a002Illumise, vec![EnergyType::Grass]);
    let mut state = game.get_state_clone();
    state.discard_piles[0].push(get_card_by_enum(CardId::B4a001Volbeat));
    game.set_state(state);
    assert_eq!(
        attack_and_read_defender_hp(&mut game, CardId::B4a002Illumise),
        210,
        "with Volbeat in the discard pile it should do 90"
    );
}
