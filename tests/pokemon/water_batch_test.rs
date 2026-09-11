use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    effects::CardEffect,
    models::{EnergyType, PlayedCard},
    test_support::{attack_action, get_initialized_game},
    Game,
};

/// Take the first option of everything an attack queued up, until the stack is empty.
fn resolve_stacked_actions(game: &mut Game) {
    while !game.get_state_clone().move_generation_stack.is_empty() {
        let (actor, choices) = game.get_state_clone().generate_possible_actions();
        game.apply_action(&Action {
            actor,
            action: choices[0].action.clone(),
            is_stack: true,
        });
    }
}

fn played_card_with_base_hp(card_id: CardId, base_hp: u32, energy: Vec<EnergyType>) -> PlayedCard {
    PlayedCard::new(get_card_by_enum(card_id), 0, base_hp, energy, false, vec![])
}

/// Araquanid's "Dangerous Claws": 60 damage, "If your opponent's Active Pokémon is a Basic
/// Pokémon, this attack does 60 more damage."
#[test]
fn test_araquanid_dangerous_claws_hits_basics_harder() {
    for (defender, expected_hp) in [
        (CardId::A1053Squirtle, 180u32), // Basic: 60 + 60
        (CardId::A1054Wartortle, 240),   // Stage 1: 60
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::A3053Araquanid).with_energy(vec![
                    EnergyType::Water,
                    EnergyType::Water,
                    EnergyType::Colorless,
                ]),
            ],
            vec![played_card_with_base_hp(defender, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A3053Araquanid, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "defender {defender:?}",
        );
    }
}

/// Ludicolo's "Rhythmic Steps": 60 damage, "If you have exactly 1, 3, or 5 cards in your hand,
/// this attack does 60 more damage."
#[test]
fn test_ludicolo_rhythmic_steps_counts_the_hand() {
    for (hand_size, expected_hp) in [(3usize, 180u32), (4, 240), (0, 240)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B1055Ludicolo)
                .with_energy(vec![EnergyType::Water, EnergyType::Water])],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A1033Charmander); hand_size];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B1055Ludicolo, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "hand of {hand_size}",
        );
    }
}

/// Luvdisc's "Paired Tackle": 30 damage, "If you have exactly 2, 4, or 6 cards in your hand,
/// this attack does 30 more damage."
#[test]
fn test_luvdisc_paired_tackle_counts_the_hand() {
    for (hand_size, expected_hp) in [(4usize, 240u32), (5, 270), (8, 270)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B1060Luvdisc)
                .with_energy(vec![EnergyType::Water, EnergyType::Water])],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A1033Charmander); hand_size];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B1060Luvdisc, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "hand of {hand_size}",
        );
    }
}

/// Team Rocket's Lapras's "Ruthless Whirlpool": 40 damage, "If this Pokémon has more Energy
/// attached than your opponent's Active Pokémon, this attack does 40 more damage."
#[test]
fn test_team_rockets_lapras_ruthless_whirlpool_compares_energy() {
    for (defender_energy, expected_hp) in [(0usize, 220u32), (2, 260)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4a013TeamRocketsLapras)
                .with_energy(vec![EnergyType::Water, EnergyType::Colorless])],
            vec![played_card_with_base_hp(
                CardId::A1053Squirtle,
                300,
                vec![EnergyType::Fire; defender_energy],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4a013TeamRocketsLapras, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "defender holding {defender_energy} Energy",
        );
    }
}

/// Wishiwashi ex's "School Storm": 30 damage, "This attack does 40 more damage for each of your
/// Benched Wishiwashi and Wishiwashi ex." Both the Basic and the ex count.
#[test]
fn test_wishiwashi_ex_school_storm_counts_the_school() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A3051WishiwashiEx).with_energy(vec![
                EnergyType::Water,
                EnergyType::Water,
                EnergyType::Water,
            ]),
            PlayedCard::from_id(CardId::A3050Wishiwashi),
            PlayedCard::from_id(CardId::A3051WishiwashiEx),
            PlayedCard::from_id(CardId::A1033Charmander),
        ],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A3051WishiwashiEx, 0),
        is_stack: false,
    });
    assert_eq!(
        game.get_state_clone().get_active(1).get_remaining_hp(),
        190,
        "30 + 40 x 2 Benched Wishiwashi",
    );
}

/// Aurorus's "Hail Prison": 90 damage, "Discard 2 [W] Energy from this Pokémon. Your opponent's
/// Active Pokémon is now Paralyzed."
#[test]
fn test_aurorus_hail_prison_pays_energy_to_paralyze() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2042Aurorus).with_energy(vec![
            EnergyType::Water,
            EnergyType::Water,
            EnergyType::Colorless,
        ])],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2042Aurorus, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 210, "90 damage");
    assert!(
        state.get_active(1).is_paralyzed(),
        "the defender is Paralyzed"
    );
    assert_eq!(
        state.get_active(0).attached_energy,
        vec![EnergyType::Colorless],
        "both [W] Energy are discarded",
    );
}

/// Lapras's "Raging Freeze": 60 damage, "If any of your Pokémon were Knocked Out by damage from
/// an attack during your opponent's last turn, your opponent's Active Pokémon is now Paralyzed."
#[test]
fn test_lapras_raging_freeze_paralyzes_only_after_a_knockout() {
    for knocked_out in [true, false] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B2b017Lapras).with_energy(vec![
                EnergyType::Water,
                EnergyType::Water,
                EnergyType::Colorless,
            ])],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        state.set_knocked_out_by_opponent_attack_last_turn(knocked_out);
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2b017Lapras, 0),
            is_stack: false,
        });

        let state = game.get_state_clone();
        assert_eq!(state.get_active(1).get_remaining_hp(), 240, "60 damage");
        assert_eq!(
            state.get_active(1).is_paralyzed(),
            knocked_out,
            "knocked out last turn = {knocked_out}",
        );
    }
}

/// Kyogre's "Tidal Blast": "Discard 3 [W] Energy from this Pokémon, and this attack does 50
/// damage to each of your opponent's Pokémon."
#[test]
fn test_kyogre_tidal_blast_hits_the_whole_board() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B4041Kyogre).with_energy(vec![
            EnergyType::Water,
            EnergyType::Water,
            EnergyType::Water,
            EnergyType::Water,
        ])],
        vec![
            played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![]),
            played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![]),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B4041Kyogre, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        250,
        "50 to the Active"
    );
    assert_eq!(
        state.in_play_pokemon[1][1]
            .as_ref()
            .expect("the Benched Pokemon is still there")
            .get_remaining_hp(),
        250,
        "50 to the Bench too",
    );
    assert_eq!(
        state.get_active(0).attached_energy,
        vec![EnergyType::Water],
        "3 of the 4 [W] Energy are discarded",
    );
}

/// Rapid Strike Urshifu's "Tornado Shot": 40 damage, "Discard a [W] Energy from this Pokémon,
/// and this attack also does 40 damage to 1 of your opponent's Benched Pokémon."
#[test]
fn test_rapid_strike_urshifu_tornado_shot_picks_a_benched_target() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B3051RapidStrikeUrshifu)
            .with_energy(vec![EnergyType::Water, EnergyType::Colorless])],
        vec![
            played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![]),
            played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![]),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B3051RapidStrikeUrshifu, 0),
        is_stack: false,
    });
    // The Benched target is the attacker's choice; take the only Benched Pokemon on offer.
    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    let choice = choices
        .into_iter()
        .find(|a| matches!(a.action, SimpleAction::ApplyDamage { .. }))
        .expect("the attack offers a Benched target");
    game.apply_action(&Action {
        actor,
        action: choice.action,
        is_stack: true,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        260,
        "40 to the Active"
    );
    assert_eq!(
        state.in_play_pokemon[1][1]
            .as_ref()
            .expect("the Benched Pokemon is still there")
            .get_remaining_hp(),
        260,
        "40 to the chosen Benched Pokemon",
    );
    assert_eq!(
        state.get_active(0).attached_energy,
        vec![EnergyType::Colorless],
        "the [W] Energy is discarded",
    );
}

/// Slowking's "Litter": "Discard up to 2 Pokémon Tool cards from your hand. This attack does 50
/// damage for each card you discarded in this way." Discarding nothing does nothing.
#[test]
fn test_slowking_litter_pays_tools_for_damage() {
    for (tools_in_hand, tools_to_discard, expected_hp) in
        [(2usize, 2usize, 200u32), (2, 0, 300), (1, 1, 250)]
    {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A4a018Slowking).with_energy(vec![EnergyType::Water])],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A2148RockyHelmet); tools_in_hand];
        state.hands[0].push(get_card_by_enum(CardId::A1053Squirtle));
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A4a018Slowking, 0),
            is_stack: false,
        });

        // How many Tools to pay is the attacker's choice; every count is offered.
        let (actor, choices) = game.get_state_clone().generate_possible_actions();
        assert_eq!(
            choices.len(),
            tools_in_hand + 1,
            "discarding 0..{tools_in_hand} Tools should all be on offer",
        );
        game.apply_action(&Action {
            actor,
            action: choices[tools_to_discard].action.clone(),
            is_stack: true,
        });
        resolve_stacked_actions(&mut game);

        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(1).get_remaining_hp(),
            expected_hp,
            "discarded {tools_to_discard} of {tools_in_hand} Tools",
        );
        assert_eq!(
            state.hands[0].len(),
            tools_in_hand + 1 - tools_to_discard,
            "the discarded Tools leave the hand",
        );
    }
}

/// Gyarados's "Wild Swing": 20 damage, "You may discard any number of your Benched [W] Pokémon.
/// This attack does 40 more damage for each Benched Pokémon you discarded in this way."
#[test]
fn test_gyarados_wild_swing_trades_the_bench_for_damage() {
    for (discarded, expected_hp) in [(0usize, 280u32), (2, 200)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::A4045Gyarados)
                    .with_energy(vec![EnergyType::Water, EnergyType::Water]),
                PlayedCard::from_id(CardId::A1053Squirtle),
                PlayedCard::from_id(CardId::A1053Squirtle),
                // A Fire Pokemon on the Bench cannot be discarded for this attack.
                PlayedCard::from_id(CardId::A1033Charmander),
            ],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A4045Gyarados, 0),
            is_stack: false,
        });

        // Every subset of the 2 discardable Benched [W] Pokemon is on offer.
        let (actor, choices) = game.get_state_clone().generate_possible_actions();
        assert_eq!(choices.len(), 4, "2^2 subsets of the Benched [W] Pokemon");
        let choice = choices
            .into_iter()
            .find(|a| match &a.action {
                SimpleAction::DiscardOwnBenchedManyThenDamage { in_play_idxs, .. } => {
                    in_play_idxs.len() == discarded
                }
                _ => false,
            })
            .expect("a choice discarding exactly that many");
        game.apply_action(&Action {
            actor,
            action: choice.action,
            is_stack: true,
        });
        resolve_stacked_actions(&mut game);

        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(1).get_remaining_hp(),
            expected_hp,
            "discarded {discarded} Benched [W] Pokemon",
        );
        assert_eq!(
            state.enumerate_bench_pokemon(0).count(),
            3 - discarded,
            "the discarded Pokemon leave the Bench",
        );
    }
}

/// Regice's "Crystal Body": "Prevent all effects of attacks used by your opponent's Pokémon done
/// to this Pokémon." Aurorus's Hail Prison still lands its 90 damage, but not the Paralysis.
#[test]
fn test_regice_crystal_body_shrugs_off_attack_effects() {
    for (defender, expect_paralyzed) in [
        (CardId::A2034Regice, false),
        (CardId::A1053Squirtle, true), // a plain defender takes the Paralysis
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B2042Aurorus).with_energy(vec![
                EnergyType::Water,
                EnergyType::Water,
                EnergyType::Colorless,
            ])],
            vec![played_card_with_base_hp(defender, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2042Aurorus, 0),
            is_stack: false,
        });

        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(1).get_remaining_hp(),
            210,
            "the damage still lands on {defender:?}",
        );
        assert_eq!(
            state.get_active(1).is_paralyzed(),
            expect_paralyzed,
            "defender {defender:?}",
        );
    }
}

/// Politoed's "Lordly Cheering": "As long as this Pokémon is on your Bench, attacks used by your
/// Pokémon that evolve from Poliwhirl do +40 damage to your opponent's Active Pokémon."
#[test]
fn test_politoed_lordly_cheering_boosts_poliwhirl_evolutions() {
    for (bencher, expected_hp) in [
        (CardId::A4040Politoed, 180u32), // 80 + 40
        (CardId::A1053Squirtle, 220),    // 80
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::A1061Poliwrath).with_energy(vec![
                    EnergyType::Water,
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                ]),
                PlayedCard::from_id(bencher),
            ],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A1061Poliwrath, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "Benched {bencher:?}",
        );
    }
}

/// Politoed only cheers from the Bench, and only for Poliwhirl's evolutions.
#[test]
fn test_politoed_lordly_cheering_is_a_bench_ability() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            // Politoed itself also evolves from Poliwhirl, but it is Active, not Benched.
            PlayedCard::from_id(CardId::A4040Politoed)
                .with_energy(vec![EnergyType::Water, EnergyType::Colorless]),
            PlayedCard::from_id(CardId::A1053Squirtle),
        ],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A4040Politoed, 0),
        is_stack: false,
    });
    assert_eq!(
        game.get_state_clone().get_active(1).get_remaining_hp(),
        240,
        "Hyper Voice's plain 60",
    );
}

/// Eiscue's "Ice Face": "If this Pokémon has full HP, it takes -40 damage from attacks from your
/// opponent's Pokémon."
#[test]
fn test_eiscue_ice_face_only_guards_a_full_hp_eiscue() {
    for (pre_damage, expected_hp) in [(0u32, 260u32), (10, 210)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        let defender =
            played_card_with_base_hp(CardId::B1080Eiscue, 300, vec![]).with_damage(pre_damage);
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::A1061Poliwrath).with_energy(vec![
                    EnergyType::Water,
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                ]),
            ],
            vec![defender],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A1061Poliwrath, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "Eiscue starting {pre_damage} damage in",
        );
    }
}

/// Samurott's "Stance": "Once during your turn, when you play this Pokémon from your hand to
/// evolve 1 of your Pokémon, you may prevent all damage from—and effects of—attacks from your
/// opponent's Pokémon done to this Pokémon until the end of your opponent's next turn."
#[test]
fn test_samurott_stance_blocks_the_next_attack() {
    // Samurott has 150 HP of its own once Dewott evolves; Mega Punch does 80 through no shield.
    for (take_stance, expected_hp) in [(true, 150u32), (false, 70)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4043Dewott)],
            vec![
                PlayedCard::from_id(CardId::A1061Poliwrath).with_energy(vec![
                    EnergyType::Water,
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                ]),
            ],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::B4044Samurott)];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: SimpleAction::Evolve {
                evolution: get_card_by_enum(CardId::B4044Samurott),
                in_play_idx: 0,
                from_deck: false,
            },
            is_stack: false,
        });
        // Taking the Stance is optional, so the ability offers it alongside a Noop.
        let (actor, choices) = game.get_state_clone().generate_possible_actions();
        let choice = choices
            .into_iter()
            .find(|a| matches!(a.action, SimpleAction::Noop) != take_stance)
            .expect("Stance is on offer after evolving");
        game.apply_action(&Action {
            actor,
            action: choice.action,
            is_stack: true,
        });

        // Hand the turn over and let the opponent swing into the shield.
        let mut state = game.get_state_clone();
        state.current_player = 1;
        game.set_state(state);
        game.apply_action(&Action {
            actor: 1,
            action: attack_action(CardId::A1061Poliwrath, 0),
            is_stack: false,
        });

        assert_eq!(
            game.get_state_clone().get_active(0).get_remaining_hp(),
            expected_hp,
            "took the Stance = {take_stance}",
        );
    }
}

/// Wishiwashi's "Call for Family": "Put 1 random Wishiwashi or Wishiwashi ex from your deck onto
/// your Bench."
#[test]
fn test_wishiwashi_call_for_family_fills_the_bench() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A3050Wishiwashi).with_energy(vec![EnergyType::Water])],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.decks[0].cards = vec![
        get_card_by_enum(CardId::A1033Charmander),
        get_card_by_enum(CardId::A3051WishiwashiEx),
    ];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A3050Wishiwashi, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert!(
        state
            .enumerate_bench_pokemon(0)
            .any(|(_, p)| p.get_name() == "Wishiwashi ex"),
        "the school comes out of the deck: {:?}",
        state.in_play_pokemon[0],
    );
}

/// Octillery's "Octazooka": 50 damage, "If the Defending Pokémon tries to use an attack, your
/// opponent flips a coin. If tails, that attack doesn't happen."
#[test]
fn test_octillery_octazooka_jams_the_defender() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A4056Octillery)
            .with_energy(vec![EnergyType::Water, EnergyType::Water])],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A4056Octillery, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 250, "50 damage");
    assert!(
        state
            .get_active(1)
            .get_active_effects()
            .iter()
            .any(|e| matches!(e, CardEffect::CoinFlipToBlockAttack)),
        "the defender now has to flip to attack",
    );
}

/// Hisuian Basculegion's "Soul Counter": 50 damage, "This attack does 50 more damage for each
/// point your opponent got during their last turn."
#[test]
fn test_hisuian_basculegion_soul_counter_answers_the_points() {
    for (points_last_turn, expected_hp) in [(0u8, 250u32), (1, 200), (2, 150)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4a018HisuianBasculegion)
                .with_energy(vec![EnergyType::Water, EnergyType::Water])],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        state.set_points_gained_last_turn(1, points_last_turn);
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4a018HisuianBasculegion, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "opponent scored {points_last_turn} last turn",
        );
    }
}

/// Tatsugiri's "Retreat Directive": "Your Active Dondozo has no Retreat Cost." The ability
/// works from the Bench, and only for Dondozo (printed cost 3).
#[test]
fn test_tatsugiri_retreat_directive_frees_dondozo() {
    for (bencher, expected) in [
        (CardId::A2b021Tatsugiri, true),
        (CardId::A1033Charmander, false),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::A2b020Dondozo),
                PlayedCard::from_id(bencher),
            ],
            vec![PlayedCard::from_id(CardId::A1053Squirtle)],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let can_retreat = actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Retreat(_)));
        assert_eq!(can_retreat, expected, "Benched {bencher:?}");
    }
}

/// The directive names Dondozo: Tatsugiri does not free anything else.
#[test]
fn test_tatsugiri_retreat_directive_only_helps_dondozo() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            // Wartortle's printed Retreat Cost is 1, so a free retreat would show up here too.
            PlayedCard::from_id(CardId::A1054Wartortle),
            PlayedCard::from_id(CardId::A2b021Tatsugiri),
        ],
        vec![PlayedCard::from_id(CardId::A1053Squirtle)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    let (_, actions) = game.get_state_clone().generate_possible_actions();
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Retreat(_))),
        "only Dondozo retreats for free",
    );
}

/// Swanna's "Feathery Cyclone": 60 damage, "Move all Energy from this Pokémon to 1 of your
/// Benched Pokémon." Every type goes, not just Water.
#[test]
fn test_swanna_feathery_cyclone_hands_off_every_energy() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A4063Swanna)
                .with_energy(vec![EnergyType::Water, EnergyType::Colorless]),
            PlayedCard::from_id(CardId::A1053Squirtle),
        ],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A4063Swanna, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 240, "60 damage");
    assert!(
        state.get_active(0).attached_energy.is_empty(),
        "Swanna keeps nothing",
    );
    let moved = &state.in_play_pokemon[0][1]
        .as_ref()
        .expect("the Benched Pokemon is there")
        .attached_energy;
    assert_eq!(moved.len(), 2, "both Energy land on the Bench: {moved:?}");
}

/// Regice's "Reflect Energy": 70 damage, "Move 2 random Energy from this Pokémon to 1 of your
/// Benched Pokémon." Only 2 of the 3 move.
#[test]
fn test_regice_reflect_energy_moves_two() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B3045Regice).with_energy(vec![
                EnergyType::Water,
                EnergyType::Water,
                EnergyType::Colorless,
            ]),
            PlayedCard::from_id(CardId::A1053Squirtle),
        ],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B3045Regice, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 230, "70 damage");
    assert_eq!(
        state.get_active(0).attached_energy.len(),
        1,
        "1 of the 3 stays behind",
    );
    assert_eq!(
        state.in_play_pokemon[0][1]
            .as_ref()
            .expect("the Benched Pokemon is there")
            .attached_energy
            .len(),
        2,
        "2 Energy move to the Bench",
    );
}

/// Veluza's "Shedding Spiral": 90 damage for [W][C][C][C], "If you have no cards in your deck,
/// this attack can be used for 1 [W] Energy."
#[test]
fn test_veluza_shedding_spiral_is_cheap_on_an_empty_deck() {
    for (deck_cards, expected) in [(0usize, true), (1, false)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B2a031Veluza).with_energy(vec![EnergyType::Water])],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.decks[0].cards = vec![get_card_by_enum(CardId::A1053Squirtle); deck_cards];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let can_attack = actions
            .iter()
            .any(|a| matches!(&a.action, SimpleAction::Attack(attack) if attack.title == "Shedding Spiral"));
        assert_eq!(can_attack, expected, "deck of {deck_cards}");
    }
}
