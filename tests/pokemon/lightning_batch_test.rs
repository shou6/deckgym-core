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

/// Rotom's "Assault Laser": 20 damage, "+30 if your opponent's Active Pokémon has
/// a Pokémon Tool attached." The existing variants look at the attacker's own
/// Tool; this one looks across the table.
#[test]
fn test_rotom_assault_laser_checks_the_defenders_tool() {
    for (with_tool, expected_hp) in [(false, 280u32), (true, 250)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        // Giant Cape would also add 20 HP and muddy the arithmetic, so use a Tool
        // that only matters for whether one is attached at all.
        let mut defender = played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]);
        if with_tool {
            defender = defender.with_tool(get_card_by_enum(CardId::A2148RockyHelmet));
        }
        state.set_board(
            vec![PlayedCard::from_id(CardId::A2062Rotom).with_energy(vec![EnergyType::Colorless])],
            vec![defender],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A2062Rotom, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "defender holding a Tool = {with_tool}"
        );
    }
}

/// Minun's "Buddy Spark": 30 damage, "If Plusle is on your Bench, this attack
/// also does 10 damage to each of your opponent's Benched Pokémon."
#[test]
fn test_minun_buddy_spark_needs_plusle() {
    for (ally, bench_hp) in [
        (CardId::A1033Charmander, 300u32),
        (CardId::B2052Plusle, 290),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B2053Minun).with_energy(vec![EnergyType::Lightning]),
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
            action: attack_action(CardId::B2053Minun, 0),
            is_stack: false,
        });

        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(1).get_remaining_hp(),
            270,
            "the Active always takes 30"
        );
        assert_eq!(
            state.in_play_pokemon[1][1]
                .as_ref()
                .expect("opponent bench")
                .get_remaining_hp(),
            bench_hp,
        );
    }
}

/// Team Rocket's Electrode's "Random Spark": "This attack does 30 damage to 1 of
/// your opponent's Pokémon." Any of them, the Active included.
#[test]
fn test_team_rockets_electrode_random_spark_can_pick_any_target() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B4a020TeamRocketsElectrode)
            .with_energy(vec![EnergyType::Lightning])],
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
        action: attack_action(CardId::B4a020TeamRocketsElectrode, 0),
        is_stack: false,
    });

    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    assert_eq!(actor, 0, "the attacker picks");
    assert_eq!(
        choices.len(),
        2,
        "both of the opponent's Pokemon are targets: {choices:?}"
    );
    game.apply_action(&choices[0].clone());

    let state = game.get_state_clone();
    let hit = state.in_play_pokemon[1]
        .iter()
        .flatten()
        .any(|p| p.get_remaining_hp() == 270);
    assert!(hit, "one of them should have taken 30");
}

/// Chinchou's "Luring Glow": "Flip a coin. If heads, switch in 1 of your
/// opponent's Benched Pokémon to the Active Spot." Same shape as Rillaboom's
/// ability, but as an attack.
#[test]
fn test_chinchou_luring_glow_is_a_coin_flip_drag() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::PA095Chinchou).with_energy(vec![EnergyType::Lightning])],
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
        action: attack_action(CardId::PA095Chinchou, 0),
        is_stack: false,
    });
    game.play_until_stable();

    // Heads drags Bulbasaur out, tails leaves Charmander in front. Either is fine;
    // what must not happen is a panic or a stuck turn.
    let name = game.get_state_clone().get_active(1).get_name();
    assert!(
        name == "Charmander" || name == "Bulbasaur",
        "unexpected Active after Luring Glow: {name}"
    );
}

/// Galvantula's "Electric Shock": 70 damage, "Discard all Energy attached to this
/// Pokémon. Your opponent's Active Pokémon is now Paralyzed."
#[test]
fn test_galvantula_electric_shock_pays_all_energy_and_paralyses() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A3b027Galvantula)
            .with_energy(vec![EnergyType::Lightning, EnergyType::Lightning])],
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
        action: attack_action(CardId::A3b027Galvantula, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 230, "70 damage");
    assert!(
        state.get_active(1).is_paralyzed(),
        "the defender should be Paralyzed"
    );
    assert!(
        state.get_active(0).attached_energy.is_empty(),
        "Galvantula pays every Energy it had"
    );
}

/// Ampharos's "Zapping Bullet": 90 damage, "1 of your opponent's Benched Pokémon
/// is chosen at random. This attack also does 20 damage to it."
#[test]
fn test_ampharos_zapping_bullet_also_hits_a_random_bench() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B1084Ampharos).with_energy(vec![
            EnergyType::Lightning,
            EnergyType::Lightning,
            EnergyType::Colorless,
        ])],
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
        action: attack_action(CardId::B1084Ampharos, 0),
        is_stack: false,
    });
    game.play_until_stable();

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        210,
        "the Active takes 90"
    );
    assert_eq!(
        state.in_play_pokemon[1][1]
            .as_ref()
            .expect("the only benched Pokemon")
            .get_remaining_hp(),
        280,
        "the single Benched Pokemon is the one chosen at random"
    );
}

/// Tapu Koko's "Volt Switch": 70 damage, "Switch this Pokémon with 1 of your
/// Benched [L] Pokémon." The switch is mandatory and only [L] Pokemon qualify.
#[test]
fn test_tapu_koko_volt_switch_only_swaps_with_lightning() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A3068TapuKoko).with_energy(vec![
                EnergyType::Lightning,
                EnergyType::Lightning,
                EnergyType::Lightning,
            ]),
            // Charmander is [R] and must not be a valid destination.
            PlayedCard::from_id(CardId::A1033Charmander),
            PlayedCard::from_id(CardId::PA095Chinchou),
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
        action: attack_action(CardId::A3068TapuKoko, 0),
        is_stack: false,
    });

    let (_, choices) = game.get_state_clone().generate_possible_actions();
    assert_eq!(
        choices.len(),
        1,
        "only the [L] Chinchou should be offered: {choices:?}"
    );
    game.apply_action(&choices[0].clone());

    assert_eq!(
        game.get_state_clone().get_active(0).get_name(),
        "Chinchou",
        "Tapu Koko swaps itself out for the [L] Pokemon"
    );
}

/// Pachirisu's "Crackling Snap": 30 damage, "Discard the top card of your deck,
/// and if that card is an Item, this attack does 20 more damage."
#[test]
fn test_pachirisu_crackling_snap_reads_the_top_card() {
    for (top, expected_hp) in [
        (CardId::PA005PokeBall, 250u32), // an Item
        (CardId::A1001Bulbasaur, 270),   // not an Item
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4054Pachirisu)
                .with_energy(vec![EnergyType::Lightning])],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        let deck_before = state.decks[0].cards.len();
        // draw() takes from the front, so the card under test goes there.
        state.decks[0].cards.insert(0, get_card_by_enum(top));
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4054Pachirisu, 0),
            is_stack: false,
        });

        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(1).get_remaining_hp(),
            expected_hp,
            "top card decides the bonus"
        );
        assert_eq!(
            state.decks[0].cards.len(),
            deck_before,
            "the card is discarded either way"
        );
    }
}

/// Toxtricity's "Vengeful Shock": 40 damage, "If any of your Pokémon were Knocked
/// Out by damage from an attack during your opponent's last turn, this attack
/// does 60 more damage, and your opponent's Active Pokémon is now Paralyzed."
/// The plain +60 version already existed; this one adds the Paralysis.
#[test]
fn test_toxtricity_vengeful_shock_adds_paralysis_after_a_knockout() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B3061Toxtricity)
            .with_energy(vec![EnergyType::Lightning, EnergyType::Lightning])],
        vec![played_card_with_base_hp(
            CardId::A1033Charmander,
            300,
            vec![],
        )],
    );
    state.current_player = 0;
    state.turn_count = 5;
    state.set_knocked_out_by_opponent_attack_last_turn(true);
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B3061Toxtricity, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        200,
        "40 + 60 after a knockout"
    );
    assert!(
        state.get_active(1).is_paralyzed(),
        "the defender should also be Paralyzed"
    );
}

/// Team Rocket's Electrode's "Destiny Burst": "If this Pokémon is in the Active Spot and is
/// Knocked Out by damage from an attack from your opponent's Pokémon, do 70 damage to the
/// Attacking Pokémon." Greninja ex's Aqua Edge does 100, so the 70 HP Electrode faints and
/// strikes back; the 200 HP variant survives and the ability stays quiet.
#[test]
fn test_team_rockets_electrode_destiny_burst() {
    for (defender_hp, expected_attacker_hp) in [(70u32, 100u32), (200, 170)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B1073GreninjaEx)
                .with_energy(vec![EnergyType::Water, EnergyType::Water])],
            vec![
                played_card_with_base_hp(CardId::B4a020TeamRocketsElectrode, defender_hp, vec![]),
                // A Benched Pokémon to promote into, so the K.O. does not end the game.
                played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B1073GreninjaEx, 0),
            is_stack: false,
        });

        assert_eq!(
            game.get_state_clone().get_active(0).get_remaining_hp(),
            expected_attacker_hp,
            "Electrode with {defender_hp} HP",
        );
    }
}

/// Destiny Burst only fires from the Active Spot: the same Electrode is Knocked Out on the
/// Bench by Alakazam's splash damage without striking back.
#[test]
fn test_team_rockets_electrode_destiny_burst_stays_quiet_on_the_bench() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![played_card_with_base_hp(
            CardId::A2b031Alakazam,
            300,
            vec![EnergyType::Psychic, EnergyType::Psychic],
        )],
        vec![
            played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
            // 20 HP so Psychic Suppression's splash damage finishes it; the splash only
            // reaches Benched Pokémon that have Energy attached.
            played_card_with_base_hp(
                CardId::B4a020TeamRocketsElectrode,
                20,
                vec![EnergyType::Lightning],
            ),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A2b031Alakazam, 0),
        is_stack: false,
    });

    assert_eq!(
        game.get_state_clone().get_active(0).get_remaining_hp(),
        300,
        "a Benched K.O. must not trigger Destiny Burst",
    );
}

/// Alolan Raichu's "Surge Surfer": "If a Stadium is in play, this Pokémon has no Retreat Cost."
/// Its printed cost is 2, so with no Stadium and no Energy it cannot move.
#[test]
fn test_alolan_raichu_surge_surfer_needs_a_stadium() {
    for with_stadium in [true, false] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B2050AlolanRaichu),
                PlayedCard::from_id(CardId::A1033Charmander),
            ],
            vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
        );
        state.current_player = 0;
        state.turn_count = 5;
        if with_stadium {
            state.active_stadium = Some(get_card_by_enum(CardId::B2153TrainingArea));
            state.active_stadium_owner = Some(1);
        }
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let can_retreat = actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Retreat(_)));
        assert_eq!(
            can_retreat, with_stadium,
            "stadium in play = {with_stadium}"
        );
    }
}

/// Boltund's "Defiant Spark": 70 damage for [L][C][C], "If this Pokémon has damage on it, this
/// attack can be used for 1 [L] Energy."
#[test]
fn test_boltund_defiant_spark_is_cheap_when_hurt() {
    for (damage_on_self, expected) in [(10u32, true), (0, false)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A4a031Boltund)
                .with_energy(vec![EnergyType::Lightning])
                .with_damage(damage_on_self)],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let can_attack = actions
            .iter()
            .any(|a| matches!(&a.action, SimpleAction::Attack(attack) if attack.title == "Defiant Spark"));
        assert_eq!(can_attack, expected, "{damage_on_self} damage on Boltund");
    }
}
