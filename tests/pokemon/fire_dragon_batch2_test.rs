use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard, StatusCondition},
    test_support::{attack_action, get_initialized_game},
};

fn played_card_with_base_hp(card_id: CardId, base_hp: u32, energy: Vec<EnergyType>) -> PlayedCard {
    PlayedCard::new(get_card_by_enum(card_id), 0, base_hp, energy, false, vec![])
}

/// Team Rocket's Magmar's "Derisive Roasting": 10 damage, "+50 for each Special
/// Condition affecting your opponent's Active Pokémon." Poison and Burn together
/// count as two.
#[test]
fn test_team_rockets_magmar_counts_each_special_condition() {
    for (conditions, expected_hp) in [
        (vec![], 290u32),
        (vec![StatusCondition::Poisoned], 240),
        (
            vec![StatusCondition::Poisoned, StatusCondition::Burned],
            190,
        ),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4a006TeamRocketsMagmar)
                .with_energy(vec![EnergyType::Colorless])],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        for condition in &conditions {
            state.apply_status_condition(1, 0, *condition);
        }
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4a006TeamRocketsMagmar, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "{} condition(s)",
            conditions.len()
        );
    }
}

/// Magmortar's "Thundering Volcano": 70 damage, "If Electivire is on your Bench,
/// this attack also does 20 damage to each of your opponent's Benched Pokémon."
#[test]
fn test_magmortar_thundering_volcano_needs_electivire_on_the_bench() {
    for (with_electivire, bench_hp) in [(false, 300u32), (true, 280)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        let ally = if with_electivire {
            CardId::B2b025Electivire
        } else {
            CardId::A1033Charmander
        };
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B2b013Magmortar).with_energy(vec![
                    EnergyType::Fire,
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                ]),
                PlayedCard::from_id(ally),
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
            action: attack_action(CardId::B2b013Magmortar, 0),
            is_stack: false,
        });

        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(1).get_remaining_hp(),
            230,
            "the Active always takes 70"
        );
        assert_eq!(
            state.in_play_pokemon[1][1]
                .as_ref()
                .expect("the opponent has a benched Pokemon")
                .get_remaining_hp(),
            bench_hp,
            "with Electivire = {with_electivire}"
        );
    }
}

/// Volcarona's "Volcanic Ash": "Discard 2 [R] Energy from this Pokémon. This
/// attack does 80 damage to 1 of your opponent's Pokémon." The target is chosen,
/// and the Energy is spent whichever target is picked.
#[test]
fn test_volcarona_volcanic_ash_spends_energy_and_hits_a_chosen_target() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1a014Volcarona).with_energy(vec![
                EnergyType::Fire,
                EnergyType::Fire,
                EnergyType::Colorless,
            ]),
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
        action: attack_action(CardId::A1a014Volcarona, 0),
        is_stack: false,
    });

    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    assert_eq!(actor, 0, "the attacker picks the target");
    assert!(
        choices
            .iter()
            .all(|c| matches!(c.action, SimpleAction::ApplyDamage { .. })),
        "every choice should be a damage target: {choices:?}"
    );
    game.apply_action(&choices[0].clone());

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(0).attached_energy.len(),
        1,
        "two [R] Energy are discarded, leaving the Colorless"
    );
    let hit = state.in_play_pokemon[1]
        .iter()
        .flatten()
        .any(|p| p.get_remaining_hp() == 220);
    assert!(hit, "one of the opponent's Pokemon should have taken 80");
}

/// Druddigon's "Giga Claw": 120 damage, "Flip 2 coins. If both of them are tails,
/// this attack does nothing." Two of the four coin outcomes are covered by
/// checking that the defender either took the full 120 or nothing at all.
#[test]
fn test_druddigon_giga_claw_is_all_or_nothing() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B1176Druddigon).with_energy(vec![
                EnergyType::Fire,
                EnergyType::Water,
                EnergyType::Colorless,
                EnergyType::Colorless,
            ]),
        ],
        vec![played_card_with_base_hp(
            CardId::A1033Charmander,
            300,
            vec![],
        )],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B1176Druddigon, 0),
        is_stack: false,
    });

    let hp = game.get_state_clone().get_active(1).get_remaining_hp();
    assert!(
        hp == 180 || hp == 300,
        "Giga Claw does 120 or nothing, got {hp}"
    );
}
