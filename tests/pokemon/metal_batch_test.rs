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

/// Bronzong's "Psychic Resonance": 50 damage, "+50 if your opponent has any [P]
/// Pokémon in play." Anywhere in play, not just the Active Spot.
#[test]
fn test_bronzong_psychic_resonance_checks_the_whole_opponent_board() {
    for (bench, expected_hp) in [
        (CardId::A1033Charmander, 250u32),
        (CardId::A1120Gastly, 200),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B3118Bronzong)
                .with_energy(vec![EnergyType::Metal, EnergyType::Colorless])],
            vec![
                played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
                PlayedCard::from_id(bench),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B3118Bronzong, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "a benched [P] Pokemon should be enough"
        );
    }
}

/// Forretress's "Enormous Explosion": 100 damage, "This Pokémon also does 100
/// damage to itself and 50 damage to all Benched Pokémon (both yours and your
/// opponent's)."
#[test]
fn test_forretress_enormous_explosion_hits_everything() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            played_card_with_base_hp(
                CardId::B2b046Forretress,
                300,
                vec![EnergyType::Metal, EnergyType::Metal, EnergyType::Metal],
            ),
            played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
        ],
        vec![
            played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
            played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![]),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2b046Forretress, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        200,
        "the opponent's Active takes 100"
    );
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        200,
        "Forretress takes 100 itself"
    );
    assert_eq!(
        state.in_play_pokemon[1][1]
            .as_ref()
            .expect("opponent bench")
            .get_remaining_hp(),
        250,
        "the opponent's bench takes 50"
    );
    assert_eq!(
        state.in_play_pokemon[0][1]
            .as_ref()
            .expect("own bench")
            .get_remaining_hp(),
        250,
        "its own bench takes 50 too"
    );
}

/// Team Rocket's Tinkaton's "Pile-Driving Hammer": 80 damage, "During your
/// opponent's next turn, attacks used by the Defending Pokémon cost 2 [C] more,
/// and its Retreat Cost is 2 [C] more."
#[test]
fn test_team_rockets_tinkaton_raises_both_costs() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B4a050TeamRocketsTinkaton).with_energy(vec![
                EnergyType::Metal,
                EnergyType::Metal,
                EnergyType::Colorless,
            ]),
        ],
        vec![
            // Bulbasaur: Vine Whip costs 1 [G], Retreat Cost 1. With +2 on each it
            // can neither attack nor retreat on one Energy.
            PlayedCard::from_id(CardId::A1001Bulbasaur).with_energy(vec![EnergyType::Grass]),
            PlayedCard::from_id(CardId::A1033Charmander),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B4a050TeamRocketsTinkaton, 0),
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });

    let (_, actions) = game.get_state_clone().generate_possible_actions();
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Attack(_))),
        "the raised attack cost should stop Vine Whip: {actions:?}"
    );
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Retreat(_))),
        "the raised Retreat Cost should stop the retreat: {actions:?}"
    );
}

/// Aegislash's "Superb Shield": 80 damage, "During your opponent's next turn,
/// this Pokémon takes -80 damage from attacks from your opponent's Pokémon ex."
/// The reduction is conditional on the attacker being a Pokemon ex.
#[test]
fn test_aegislash_superb_shield_only_blunts_ex_attackers() {
    // Charizard ex's Slash does 60; with -80 Aegislash should take nothing.
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B2120Aegislash).with_energy(vec![
                EnergyType::Metal,
                EnergyType::Metal,
                EnergyType::Metal,
            ]),
        ],
        vec![played_card_with_base_hp(
            CardId::A1036CharizardEx,
            300,
            vec![
                EnergyType::Fire,
                EnergyType::Colorless,
                EnergyType::Colorless,
            ],
        )],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2120Aegislash, 0),
        is_stack: false,
    });
    let aegislash_hp = game.get_state_clone().get_active(0).get_remaining_hp();
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 1,
        action: attack_action(CardId::A1036CharizardEx, 0),
        is_stack: false,
    });

    // The engine subtracts reductions first and adds Weakness afterwards, so
    // Slash's 60 is wiped out by the -80 and only Aegislash's [R] Weakness (+20)
    // gets through.
    assert_eq!(
        game.get_state_clone().get_active(0).get_remaining_hp(),
        aegislash_hp - 20,
        "the -80 absorbs Slash's 60, leaving only the Weakness bonus"
    );
}

/// A non-ex attacker is unaffected by Superb Shield.
#[test]
fn test_aegislash_superb_shield_ignores_non_ex_attackers() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B2120Aegislash).with_energy(vec![
                EnergyType::Metal,
                EnergyType::Metal,
                EnergyType::Metal,
            ]),
        ],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![EnergyType::Grass],
        )],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2120Aegislash, 0),
        is_stack: false,
    });
    let aegislash_hp = game.get_state_clone().get_active(0).get_remaining_hp();
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 1,
        action: attack_action(CardId::A1001Bulbasaur, 0),
        is_stack: false,
    });

    assert_eq!(
        game.get_state_clone().get_active(0).get_remaining_hp(),
        aegislash_hp - 40,
        "Vine Whip is not from an ex, so its 40 lands in full"
    );
}
