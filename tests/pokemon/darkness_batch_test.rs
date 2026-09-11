use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
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

/// Seviper's "Fateful Fang": 40 damage, "If your opponent's Active Pokémon is Zangoose, this
/// attack does 40 more damage." The feud is spelled out by name.
#[test]
fn test_seviper_fateful_fang_hunts_zangoose() {
    for (defender, expected_hp) in [
        (CardId::A4a065Zangoose, 220u32), // 40 + 40
        (CardId::A1001Bulbasaur, 260),    // 40
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A4a048Seviper)
                .with_energy(vec![EnergyType::Darkness, EnergyType::Colorless])],
            vec![played_card_with_base_hp(defender, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A4a048Seviper, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "defender {defender:?}",
        );
    }
}

/// Weavile's "Raid": 40 damage, "If this Pokémon evolved from Sneasel during this turn, this
/// attack does 20 more damage."
#[test]
fn test_weavile_raid_rewards_evolving_this_turn() {
    for (evolve_now, expected_hp) in [(true, 240u32), (false, 260)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        if evolve_now {
            state.set_board(
                vec![PlayedCard::from_id(CardId::B3a038Sneasel)
                    .with_energy(vec![EnergyType::Darkness])],
                vec![played_card_with_base_hp(
                    CardId::A1001Bulbasaur,
                    300,
                    vec![],
                )],
            );
            state.hands[0] = vec![get_card_by_enum(CardId::B3a039Weavile)];
        } else {
            state.set_board(
                vec![PlayedCard::from_id(CardId::B3a039Weavile)
                    .with_energy(vec![EnergyType::Darkness])],
                vec![played_card_with_base_hp(
                    CardId::A1001Bulbasaur,
                    300,
                    vec![],
                )],
            );
        }
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        if evolve_now {
            game.apply_action(&Action {
                actor: 0,
                action: SimpleAction::Evolve {
                    evolution: get_card_by_enum(CardId::B3a039Weavile),
                    in_play_idx: 0,
                    from_deck: false,
                },
                is_stack: false,
            });
        }
        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B3a039Weavile, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "evolved this turn = {evolve_now}",
        );
    }
}

/// Swalot's "Swallow Up": 30 damage, "If your opponent's Active Pokémon has less remaining HP
/// than this Pokémon, this attack does 80 more damage." Swalot has 120 HP printed.
#[test]
fn test_swalot_swallow_up_picks_on_the_weaker() {
    for (defender_hp, expected_hp) in [(60u32, 0u32), (300, 270)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4099Swalot)
                .with_energy(vec![EnergyType::Darkness, EnergyType::Colorless])],
            vec![
                played_card_with_base_hp(CardId::A1001Bulbasaur, defender_hp, vec![]),
                PlayedCard::from_id(CardId::A1001Bulbasaur),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4099Swalot, 0),
            is_stack: false,
        });
        let state = game.get_state_clone();
        let remaining = state.in_play_pokemon[1][0]
            .as_ref()
            .map_or(0, |p| p.get_remaining_hp());
        assert_eq!(
            remaining, expected_hp,
            "defender with {defender_hp} HP (110 damage knocks the 60 HP one out)",
        );
    }
}

/// Team Rocket's Muk's "Poison Absorption": 80 damage, "If your opponent's Active Pokémon is
/// Poisoned, heal 60 damage from this Pokémon."
#[test]
fn test_team_rockets_muk_poison_absorption_heals_off_poison() {
    for (poisoned, expected_self_hp) in [(true, 120u32), (false, 60)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4a041TeamRocketsMuk)
                .with_energy(vec![
                    EnergyType::Darkness,
                    EnergyType::Darkness,
                    EnergyType::Colorless,
                ])
                .with_damage(60)],
            vec![played_card_with_base_hp(
                CardId::A1001Bulbasaur,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        if poisoned {
            state.apply_status_condition(1, 0, deckgym::models::StatusCondition::Poisoned);
        }
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4a041TeamRocketsMuk, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(0).get_remaining_hp(),
            expected_self_hp,
            "defender Poisoned = {poisoned} (Muk has 120 HP, 60 damage on it)",
        );
    }
}

/// Toxicroak's "Toxic" and Toxapex's "Severe Poison": the Poison they leave behind bites for 20
/// and 40 instead of the usual 10.
#[test]
fn test_heavy_poison_replaces_the_usual_amount() {
    for (attacker, defender_hp_after_checkup) in [
        (CardId::A2a052Toxicroak, 280u32), // 20 per checkup
        (CardId::B3b047ToxapEx, 260),      // 40 per checkup
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(attacker)
                .with_energy(vec![EnergyType::Darkness, EnergyType::Colorless])],
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
            action: attack_action(attacker, 0),
            is_stack: false,
        });
        assert!(
            game.get_state_clone().get_active(1).is_poisoned(),
            "the defender is Poisoned by {attacker:?}",
        );

        // End the turn so the Pokémon Checkup applies the poison damage.
        game.apply_action(&Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            defender_hp_after_checkup,
            "poison from {attacker:?}",
        );
    }
}

/// Alolan Raticate's "Scrounge-and-Scarf": 50 damage, "Discard a random Item card from your
/// opponent's hand." Only Items go.
#[test]
fn test_alolan_raticate_scrounge_takes_an_item() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A3107AlolanRaticate)
            .with_energy(vec![EnergyType::Darkness, EnergyType::Darkness])],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![],
        )],
    );
    state.hands[1] = vec![
        get_card_by_enum(CardId::PA001Potion),    // Item
        get_card_by_enum(CardId::A1001Bulbasaur), // not an Item
    ];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A3107AlolanRaticate, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 250, "50 damage");
    assert_eq!(state.hands[1].len(), 1, "one card leaves the hand");
    assert_eq!(
        state.hands[1][0].get_name(),
        "Bulbasaur",
        "the Item is the one that goes",
    );
}

/// Alolan Meowth's "Meddle": "Discard a random Pokémon Tool card from your opponent's hand."
#[test]
fn test_alolan_meowth_meddle_takes_a_tool() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A3a037AlolanMeowth).with_energy(vec![EnergyType::Darkness])
        ],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![],
        )],
    );
    state.hands[1] = vec![
        get_card_by_enum(CardId::PA001Potion), // Item, not a Tool
        get_card_by_enum(CardId::A2148RockyHelmet),
    ];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A3a037AlolanMeowth, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.hands[1].len(), 1, "one card leaves the hand");
    assert_eq!(
        state.hands[1][0].get_name(),
        "Potion",
        "the Tool is the one that goes",
    );
}

/// Purrloin's "Playful Knockdown": "Discard all Pokémon Tools from your opponent's Active
/// Pokémon."
#[test]
fn test_purrloin_playful_knockdown_strips_the_tool() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A3b045Purrloin).with_energy(vec![EnergyType::Darkness])],
        vec![
            played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![])
                .with_tool(get_card_by_enum(CardId::A2148RockyHelmet)),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A3b045Purrloin, 0),
        is_stack: false,
    });

    assert!(
        game.get_state_clone().get_active(1).attached_tool.is_none(),
        "the Tool is knocked off",
    );
}

/// Alolan Muk ex's "Chemical Panic": 80 damage, and one Special Condition the defender does not
/// already have is picked at random.
#[test]
fn test_alolan_muk_ex_chemical_panic_adds_one_condition() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A3111AlolanMukEx).with_energy(vec![
                EnergyType::Darkness,
                EnergyType::Darkness,
                EnergyType::Colorless,
            ]),
        ],
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
        action: attack_action(CardId::A3111AlolanMukEx, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 220, "80 damage");
    let defender = state.get_active(1);
    let conditions = [
        defender.is_poisoned(),
        defender.is_paralyzed(),
        defender.is_asleep(),
        defender.is_burned(),
        defender.is_confused(),
    ];
    assert_eq!(
        conditions.iter().filter(|applied| **applied).count(),
        1,
        "exactly one Special Condition lands",
    );
}

/// Grafaiai's "Poison Coating": "Once during your turn, you may flip a coin. If heads, your
/// opponent's Active Pokémon is now Poisoned."
#[test]
fn test_grafaiai_poison_coating_is_a_coin_flip() {
    let mut poisoned = 0;
    let seed_count = 24u64;
    for seed in 0..seed_count {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A2b051Grafaiai)],
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
            action: SimpleAction::UseAbility { in_play_idx: 0 },
            is_stack: false,
        });
        if game.get_state_clone().get_active(1).is_poisoned() {
            poisoned += 1;
        }
    }
    assert!(
        poisoned > 0 && poisoned < seed_count,
        "the Poison should follow a coin flip (poisoned {poisoned}/{seed_count})",
    );
}

/// Liepard's "Snatch and Flee": 60 damage, "Your opponent reveals a random card from their hand
/// and shuffles it into their deck. Shuffle this Pokémon into your deck."
#[test]
fn test_liepard_snatch_and_flee_leaves_the_board() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B1a048Liepard).with_energy(vec![EnergyType::Darkness]),
            PlayedCard::from_id(CardId::A1001Bulbasaur),
        ],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![],
        )],
    );
    state.hands[1] = vec![get_card_by_enum(CardId::PA001Potion)];
    state.decks[1].cards = vec![];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B1a048Liepard, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 240, "60 damage");
    assert!(state.hands[1].is_empty(), "the hand card is shuffled away");
    assert_eq!(state.decks[1].cards.len(), 1, "and lands in the deck");
    assert!(
        state
            .enumerate_in_play_pokemon(0)
            .all(|(_, p)| p.get_name() != "Liepard"),
        "Liepard shuffles itself back into the deck",
    );
}

/// Kingambit's "Overlord's Blade": 60 damage, "This attack does 40 more damage for each time your
/// Pokémon have been Knocked Out during this game."
#[test]
fn test_kingambit_overlords_blade_counts_your_losses() {
    for (losses, expected_hp) in [(0u8, 240u32), (2, 160)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B3a043Kingambit)
                .with_energy(vec![EnergyType::Darkness, EnergyType::Darkness])],
            vec![played_card_with_base_hp(
                CardId::A1001Bulbasaur,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        state.set_own_knockouts_this_game(0, losses);
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B3a043Kingambit, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "{losses} of your Pokemon knocked out so far",
        );
    }
}

/// Spiritomb's "Final Scream": "If this Pokémon is in the Active Spot and is Knocked Out by
/// damage from an attack from your opponent's Pokémon, do 10 damage to each of your opponent's
/// Pokémon." Same trigger as Destiny Burst, but it splashes.
#[test]
fn test_spiritomb_final_scream_splashes_on_the_way_out() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A1061Poliwrath).with_energy(vec![
                EnergyType::Water,
                EnergyType::Colorless,
                EnergyType::Colorless,
            ]),
            played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![]),
        ],
        vec![
            played_card_with_base_hp(CardId::B2103Spiritomb, 70, vec![]),
            PlayedCard::from_id(CardId::A1001Bulbasaur),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A1061Poliwrath, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        140,
        "10 to the attacker (Poliwrath has 150 HP)",
    );
    assert_eq!(
        state.in_play_pokemon[0][1]
            .as_ref()
            .expect("the Benched Pokemon is there")
            .get_remaining_hp(),
        290,
        "and 10 to its Bench",
    );
}

/// Malamar's "Evolution Jammer": 40 damage, "During your opponent's next turn, they can't play
/// any Pokémon from their hand to evolve their Pokémon."
#[test]
fn test_malamar_evolution_jammer_blocks_evolving() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B3112Malamar)
            .with_energy(vec![EnergyType::Darkness, EnergyType::Colorless])],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.hands[1] = vec![get_card_by_enum(CardId::A1054Wartortle)];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B3112Malamar, 0),
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });

    let (actor, actions) = game.get_state_clone().generate_possible_actions();
    assert_eq!(actor, 1, "it is the opponent's turn");
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Evolve { .. })),
        "evolving is off the table: {actions:?}",
    );
}

/// Tyranitar's "Energy Plunder": "Once during your turn, you may move all [D] Energy from each of
/// your Pokémon to this Pokémon."
#[test]
fn test_tyranitar_energy_plunder_gathers_the_darkness() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A4119Tyranitar).with_energy(vec![EnergyType::Darkness]),
            PlayedCard::from_id(CardId::A1001Bulbasaur)
                .with_energy(vec![EnergyType::Darkness, EnergyType::Grass]),
            PlayedCard::from_id(CardId::A1001Bulbasaur).with_energy(vec![EnergyType::Darkness]),
        ],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::UseAbility { in_play_idx: 0 },
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(0).attached_energy.len(),
        3,
        "all 3 [D] Energy gather on Tyranitar",
    );
    assert_eq!(
        state.in_play_pokemon[0][1]
            .as_ref()
            .expect("the Benched Pokemon is there")
            .attached_energy,
        vec![EnergyType::Grass],
        "the [G] Energy stays where it is",
    );
}

/// Sableye's "Jeweled Gift": "Take a random Energy from among [G], [R], [W], [L], [P], [F], [D],
/// and [M] Energy from your Energy Zone and attach it to 1 of your Benched Pokémon."
#[test]
fn test_sableye_jeweled_gift_attaches_to_the_bench() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B3a040Sableye).with_energy(vec![EnergyType::Colorless]),
            PlayedCard::from_id(CardId::A1001Bulbasaur),
        ],
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
        action: attack_action(CardId::B3a040Sableye, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    let bench = game.get_state_clone().in_play_pokemon[0][1]
        .as_ref()
        .expect("the Benched Pokemon is there")
        .attached_energy
        .clone();
    assert_eq!(bench.len(), 1, "one Energy lands on the Bench");
    assert_ne!(
        bench[0],
        EnergyType::Colorless,
        "and it is one of the 8 typed Energy",
    );
}

/// Guzzlord's "Breakcore": "Flip a coin. If heads, discard your opponent's Active Pokémon."
#[test]
fn test_guzzlord_breakcore_discards_on_heads() {
    let mut discarded = 0;
    let seed_count = 24u64;
    for seed in 0..seed_count {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B2109Guzzlord).with_energy(vec![
                EnergyType::Darkness,
                EnergyType::Darkness,
                EnergyType::Darkness,
                EnergyType::Colorless,
            ])],
            vec![
                played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![]),
                PlayedCard::from_id(CardId::A1001Bulbasaur),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2109Guzzlord, 0),
            is_stack: false,
        });
        if game.get_state_clone().in_play_pokemon[1][0].is_none() {
            discarded += 1;
        }
    }
    assert!(
        discarded > 0 && discarded < seed_count,
        "the discard should follow a coin flip (discarded {discarded}/{seed_count})",
    );
}

/// Glimmora's "Shattering Crystal": "When this Pokémon is Knocked Out, flip a coin. If heads,
/// your opponent can't get any points for it."
#[test]
fn test_glimmora_shattering_crystal_can_deny_the_point() {
    let mut denied = 0;
    let seed_count = 24u64;
    for seed in 0..seed_count {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::A1061Poliwrath).with_energy(vec![
                    EnergyType::Water,
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                ]),
            ],
            vec![
                played_card_with_base_hp(CardId::B3a045Glimmora, 70, vec![]),
                PlayedCard::from_id(CardId::A1001Bulbasaur),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A1061Poliwrath, 0),
            is_stack: false,
        });
        let state = game.get_state_clone();
        assert!(
            state.in_play_pokemon[1][0].is_none(),
            "Glimmora is Knocked Out either way (seed {seed})",
        );
        if state.points[0] == 0 {
            denied += 1;
        }
    }
    assert!(
        denied > 0 && denied < seed_count,
        "the denial should follow a coin flip (denied {denied}/{seed_count})",
    );
}

/// Grafaiai's "Colorful Attack": 30 damage, "If your Pokémon in play have 3 or more different
/// types of Energy attached, this attack does 60 more damage." The count is across the board,
/// not just the attacker (a different Grafaiai from the one with Poison Coating).
#[test]
fn test_grafaiai_colorful_attack_counts_the_whole_board() {
    for (bench_energy, expected_hp) in [
        (vec![EnergyType::Water, EnergyType::Grass], 210u32), // 3 types: 30 + 60
        (vec![EnergyType::Water], 270),                       // 2 types: 30
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B2a070Grafaiai).with_energy(vec![EnergyType::Darkness]),
                PlayedCard::from_id(CardId::A1001Bulbasaur).with_energy(bench_energy.clone()),
            ],
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
            action: attack_action(CardId::B2a070Grafaiai, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "Bench holding {bench_energy:?}",
        );
    }
}
