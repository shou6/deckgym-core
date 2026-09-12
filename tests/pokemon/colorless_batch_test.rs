use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::{attack_action, get_initialized_game},
    Game,
};

fn played_card_with_base_hp(card_id: CardId, base_hp: u32, energy: Vec<EnergyType>) -> PlayedCard {
    PlayedCard::new(get_card_by_enum(card_id), 0, base_hp, energy, false, vec![])
}

/// Take the first option of everything an attack queued up, until the stack is empty.
#[allow(dead_code)]
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

/// Kangaskhan's "Cross-Cut": 20 damage, "If your opponent's Active Pokémon is an Evolution
/// Pokémon, this attack does 40 more damage." The mirror of Araquanid's Dangerous Claws.
#[test]
fn test_kangaskhan_cross_cut_hits_evolutions_harder() {
    for (defender, expected_hp) in [
        (CardId::A1054Wartortle, 240u32), // Stage 1: 20 + 40
        (CardId::A1053Squirtle, 280),     // Basic: 20
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A4133Kangaskhan)
                .with_energy(vec![EnergyType::Colorless])],
            vec![played_card_with_base_hp(defender, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A4133Kangaskhan, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "defender {defender:?}",
        );
    }
}

/// Kecleon's "Samesies Slap": 20 damage, "+30 if this Pokémon and your opponent's Active Pokémon
/// have 1 or more of the same type of Energy attached." Enamorus's clause with smaller numbers.
#[test]
fn test_kecleon_samesies_slap_needs_a_shared_type() {
    for (defender_energy, expected_hp) in [(EnergyType::Water, 250u32), (EnergyType::Fire, 280)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4136Kecleon).with_energy(vec![EnergyType::Water])],
            vec![played_card_with_base_hp(
                CardId::A1053Squirtle,
                300,
                vec![defender_energy],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4136Kecleon, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "defender holding {defender_energy:?}",
        );
    }
}

/// Staraptor's "Defensive Whirlwind": "This Pokémon takes -30 damage from attacks from [F]
/// Pokémon." Machamp's Seismic Toss does 100, so the reduction is visible either way.
#[test]
fn test_staraptor_defensive_whirlwind_softens_fighting() {
    for (attacker, energy, expected_hp) in [
        (
            CardId::A1145Machamp,
            vec![
                EnergyType::Fighting,
                EnergyType::Fighting,
                EnergyType::Fighting,
            ],
            230u32, // [F]: 100 - 30
        ),
        (
            CardId::A1061Poliwrath,
            vec![
                EnergyType::Water,
                EnergyType::Colorless,
                EnergyType::Colorless,
            ],
            220, // [W]: the full 80
        ),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(attacker).with_energy(energy)],
            vec![played_card_with_base_hp(
                CardId::PA047Staraptor,
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
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "attacker {attacker:?}",
        );
    }
}

/// Hoothoot's "Insomnia": "This Pokémon can't be Asleep." Other conditions still land.
#[test]
fn test_hoothoot_insomnia_blocks_only_sleep() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A4091Musharna)
            .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless])],
        vec![played_card_with_base_hp(CardId::A4140Hoothoot, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    // Dream Dance puts both Active Pokemon to sleep.
    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A4091Musharna, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert!(!state.get_active(1).is_asleep(), "Hoothoot stays awake");
    assert!(state.get_active(0).is_asleep(), "Musharna does not");
}

/// Watchog's "Psych Up": 30 damage, "During your next turn, this Pokémon's Psych Up attack does
/// +30 damage."
#[test]
fn test_watchog_psych_up_builds_on_itself() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B3136Watchog).with_energy(vec![EnergyType::Colorless])],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B3136Watchog, 0),
        is_stack: false,
    });
    assert!(
        game.get_state_clone()
            .get_active(0)
            .get_active_effects()
            .iter()
            .any(|e| matches!(
                e,
                deckgym::effects::CardEffect::IncreasedDamageForAttack { amount: 30, .. }
            )),
        "the next Psych Up is stronger",
    );
}

/// Aipom's "Imitate": "Draw cards until you have the same number of cards in your hand as your
/// opponent."
#[test]
fn test_aipom_imitate_matches_the_opponents_hand() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A4142Aipom).with_energy(vec![EnergyType::Colorless])],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.hands[0] = vec![get_card_by_enum(CardId::A1053Squirtle)];
    state.hands[1] = vec![get_card_by_enum(CardId::A1053Squirtle); 4];
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1001Bulbasaur); 5];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A4142Aipom, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.hands[0].len(), 4, "3 cards are drawn to match");
    assert_eq!(state.decks[0].cards.len(), 2, "and leave the deck");
}

/// Bewear's "Superpowered Hug": "Flip 2 coins. If both of them are heads, your opponent's Active
/// Pokémon is Knocked Out." Scream Tail's clause, worded as a knockout.
#[test]
fn test_bewear_superpowered_hug_needs_two_heads() {
    let mut knocked_out = 0;
    let seed_count = 40u64;
    for seed in 0..seed_count {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A3a058Bewear).with_energy(vec![
                EnergyType::Colorless,
                EnergyType::Colorless,
                EnergyType::Colorless,
            ])],
            vec![
                played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![]),
                PlayedCard::from_id(CardId::A1053Squirtle),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A3a058Bewear, 0),
            is_stack: false,
        });
        if game.get_state_clone().in_play_pokemon[1][0].is_none() {
            knocked_out += 1;
        }
    }
    assert!(
        knocked_out > 0 && knocked_out < seed_count / 2,
        "both coins must come up heads (knocked out {knocked_out}/{seed_count})",
    );
}

/// Porygon's "Data Scan": "Once during your turn, you may look at the top card of your deck."
/// Information this engine already has, so it changes nothing.
#[test]
fn test_porygon_data_scan_is_information_only() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A1209Porygon)],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1001Bulbasaur); 3];
    state.hands[0].clear();
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::UseAbility { in_play_idx: 0 },
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.decks[0].cards.len(), 3, "the deck is untouched");
    assert!(state.hands[0].is_empty(), "and nothing is drawn");
}

/// Bidoof's "Super Fang": "Halve your opponent's Active Pokémon's remaining HP, rounded down."
#[test]
fn test_bidoof_super_fang_halves_what_is_left() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A2135Bidoof)
            .with_energy(vec![EnergyType::Colorless, EnergyType::Colorless])],
        // 250 HP left of 300, so 125 damage lands and 125 remains.
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![]).with_damage(50)],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A2135Bidoof, 0),
        is_stack: false,
    });
    assert_eq!(
        game.get_state_clone().get_active(1).get_remaining_hp(),
        125,
        "half of the 250 that was left, rounded down",
    );
}

/// Fan Rotom's "Spin Storm": "Flip a coin. If heads, put your opponent's Active Pokémon into
/// their hand."
#[test]
fn test_fan_rotom_spin_storm_can_bounce_the_active() {
    let mut bounced = 0;
    let seed_count = 24u64;
    for seed in 0..seed_count {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A2142FanRotom)
                .with_energy(vec![EnergyType::Colorless, EnergyType::Colorless])],
            vec![
                played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![]),
                PlayedCard::from_id(CardId::A1053Squirtle),
            ],
        );
        state.hands[1].clear();
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A2142FanRotom, 0),
            is_stack: false,
        });
        resolve_stacked_actions(&mut game);
        if !game.get_state_clone().hands[1].is_empty() {
            bounced += 1;
        }
    }
    assert!(
        bounced > 0 && bounced < seed_count,
        "the bounce follows a coin flip (bounced {bounced}/{seed_count})",
    );
}

/// Oranguru's "Primate's Trap": 40 damage, "During your opponent's next turn, attacks used by the
/// Defending Pokémon cost 1 [C] more, and its Retreat Cost is 1 [C] more."
#[test]
fn test_oranguru_primates_trap_taxes_both() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A3140Oranguru)
            .with_energy(vec![EnergyType::Colorless, EnergyType::Colorless])],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A3140Oranguru, 0),
        is_stack: false,
    });

    let effects = game.get_state_clone().get_active(1).get_active_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            deckgym::effects::CardEffect::IncreasedAttackCost { amount: 1 }
        )),
        "attacks cost 1 more: {effects:?}",
    );
    assert!(
        effects.iter().any(|e| matches!(
            e,
            deckgym::effects::CardEffect::IncreasedRetreatCost { amount: 1 }
        )),
        "and retreating does too: {effects:?}",
    );
}

/// Maushold's "Family Beatdown": "Flip a coin for each Tandemaus and Maushold you have in play.
/// This attack does 60 damage for each heads."
#[test]
fn test_maushold_family_beatdown_flips_per_family_member() {
    let mut totals = std::collections::HashSet::new();
    for seed in 0..24u64 {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B2143Maushold).with_energy(vec![EnergyType::Colorless]),
                PlayedCard::from_id(CardId::B2142Tandemaus),
                // Not family: no coin for this one.
                PlayedCard::from_id(CardId::A1053Squirtle),
            ],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2143Maushold, 0),
            is_stack: false,
        });
        totals.insert(300 - game.get_state_clone().get_active(1).get_remaining_hp());
    }
    assert_eq!(
        totals.iter().copied().max().unwrap(),
        120,
        "2 family members means at most 2 heads: {totals:?}",
    );
    assert!(
        totals.contains(&0),
        "and both tails means nothing: {totals:?}",
    );
}

/// Ambipom's "Catching Tail": "Once during your turn, you may put a random Pokémon Tool card from
/// your deck into your hand."
#[test]
fn test_ambipom_catching_tail_fetches_a_tool() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A3b059Ambipom)],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.hands[0].clear();
    state.decks[0].cards = vec![
        get_card_by_enum(CardId::A1001Bulbasaur),
        get_card_by_enum(CardId::A2148RockyHelmet),
    ];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::UseAbility { in_play_idx: 0 },
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.hands[0].len(), 1, "one card comes to hand");
    assert_eq!(
        state.hands[0][0].get_name(),
        "Rocky Helmet",
        "and it is the Tool, not the Pokemon",
    );
}

/// Swellow's "Repelling Wind": "Once during your turn, you may switch out your opponent's Active
/// Basic Pokémon to the Bench."
#[test]
fn test_swellow_repelling_wind_only_moves_basics() {
    for (defender, expect_offered) in [
        (CardId::A1053Squirtle, true),
        (CardId::A1054Wartortle, false),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B2133Swellow)],
            vec![
                played_card_with_base_hp(defender, 300, vec![]),
                PlayedCard::from_id(CardId::A1053Squirtle),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: SimpleAction::UseAbility { in_play_idx: 0 },
            is_stack: false,
        });
        let offered = !game.get_state_clone().move_generation_stack.is_empty();
        assert_eq!(offered, expect_offered, "opposing Active {defender:?}");
    }
}

/// Delcatty's "Search for Friends": "Once during your turn, when you play this Pokémon from your
/// hand to evolve 1 of your Pokémon, you may put a Supporter card from your discard pile into
/// your hand."
#[test]
fn test_delcatty_search_for_friends_recovers_a_supporter() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B1193Skitty)],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.hands[0] = vec![get_card_by_enum(CardId::B1194Delcatty)];
    state.discard_piles[0] = vec![
        get_card_by_enum(CardId::PA001Potion), // Item, not a Supporter
        get_card_by_enum(CardId::A1223Giovanni),
    ];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Evolve {
            evolution: get_card_by_enum(CardId::B1194Delcatty),
            in_play_idx: 0,
            from_deck: false,
        },
        is_stack: false,
    });
    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    let choice = choices
        .into_iter()
        .find(|a| !matches!(a.action, SimpleAction::Noop))
        .expect("Search for Friends is on offer after evolving");
    game.apply_action(&Action {
        actor,
        action: choice.action,
        is_stack: true,
    });

    let state = game.get_state_clone();
    assert!(
        state.hands[0].iter().any(|c| c.get_name() == "Giovanni"),
        "the Supporter comes back: {:?}",
        state.hands[0],
    );
    assert_eq!(
        state.discard_piles[0].len(),
        1,
        "and leaves the discard pile"
    );
}

/// Raticate's "Treasure Collecting": "Once during your turn, when you play this Pokémon from your
/// hand to evolve 1 of your Pokémon, you may look at the top 4 cards of your deck and put all
/// Item cards you find there into your hand."
#[test]
fn test_raticate_treasure_collecting_takes_every_item() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B4129Rattata)],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.hands[0] = vec![get_card_by_enum(CardId::B4130Raticate)];
    state.decks[0].cards = vec![
        get_card_by_enum(CardId::PA001Potion),
        get_card_by_enum(CardId::A1001Bulbasaur),
        get_card_by_enum(CardId::PA005PokeBall),
        get_card_by_enum(CardId::A1001Bulbasaur),
        // The 5th card is out of reach.
        get_card_by_enum(CardId::PA001Potion),
    ];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Evolve {
            evolution: get_card_by_enum(CardId::B4130Raticate),
            in_play_idx: 0,
            from_deck: false,
        },
        is_stack: false,
    });
    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    let choice = choices
        .into_iter()
        .find(|a| !matches!(a.action, SimpleAction::Noop))
        .expect("Treasure Collecting is on offer after evolving");
    game.apply_action(&Action {
        actor,
        action: choice.action,
        is_stack: true,
    });

    let state = game.get_state_clone();
    let items = state.hands[0]
        .iter()
        .filter(|c| c.get_name() == "Potion" || c.get_name() == "Poké Ball")
        .count();
    assert_eq!(
        items, 2,
        "both Items in the top 4 come to hand: {:?}",
        state.hands[0]
    );
    assert_eq!(
        state.decks[0].cards.len(),
        3,
        "the rest go back into the deck"
    );
}

/// Regigigas's "Seal of Antiquity": "If you don't have Regirock, Regice, and Registeel on your
/// Bench, this Pokémon can't attack."
#[test]
fn test_regigigas_seal_of_antiquity_needs_all_three() {
    for (bench, expected) in [
        (
            vec![
                CardId::A2087Regirock,
                CardId::A2034Regice,
                CardId::A2112Registeel,
            ],
            true,
        ),
        (
            vec![
                CardId::A2087Regirock,
                CardId::A2034Regice,
                CardId::A1053Squirtle,
            ],
            false,
        ),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B3134Regigigas).with_energy(vec![
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                ]),
                PlayedCard::from_id(bench[0]),
                PlayedCard::from_id(bench[1]),
                PlayedCard::from_id(bench[2]),
            ],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let can_attack = actions
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Attack(_)));
        assert_eq!(can_attack, expected, "Bench {bench:?}");
    }
}

/// Purugly's "Interrupt": 60 damage, "Your opponent reveals their hand. Choose a card you find
/// there and shuffle it into your opponent's deck."
#[test]
fn test_purugly_interrupt_sends_a_card_back() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A2140Purugly).with_energy(vec![
            EnergyType::Colorless,
            EnergyType::Colorless,
            EnergyType::Colorless,
        ])],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.hands[1] = vec![
        get_card_by_enum(CardId::PA001Potion),
        get_card_by_enum(CardId::A1001Bulbasaur),
    ];
    state.decks[1].cards = vec![];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A2140Purugly, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 240, "60 damage");
    assert_eq!(state.hands[1].len(), 1, "one card leaves the hand");
    assert_eq!(state.decks[1].cards.len(), 1, "and goes into the deck");
}

/// Smeargle's "Splatter Coating": 50 damage, "Change the type of a random Energy attached to your
/// opponent's Active Pokémon to 1 of the following at random."
#[test]
fn test_smeargle_splatter_coating_repaints_an_energy() {
    let mut seen = std::collections::HashSet::new();
    for seed in 0..24u64 {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A4148Smeargle)
                .with_energy(vec![EnergyType::Colorless, EnergyType::Colorless])],
            vec![played_card_with_base_hp(
                CardId::A1053Squirtle,
                300,
                vec![EnergyType::Water],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A4148Smeargle, 0),
            is_stack: false,
        });
        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(1).attached_energy.len(),
            1,
            "the Energy is repainted, not removed",
        );
        seen.insert(state.get_active(1).attached_energy[0]);
    }
    assert!(seen.len() > 1, "the new type is drawn at random: {seen:?}",);
    assert!(
        !seen.contains(&EnergyType::Colorless),
        "and it is one of the 8 typed Energy: {seen:?}",
    );
}

/// Dudunsparce's "Sudden Drilling": 60 damage, "If this Pokémon evolved from Dunsparce during
/// this turn, discard 2 random Energy from your opponent's Active Pokémon."
#[test]
fn test_dudunsparce_sudden_drilling_needs_a_fresh_evolution() {
    for (evolve_now, expected_energy) in [(true, 1usize), (false, 3)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        let defender = played_card_with_base_hp(
            CardId::A1053Squirtle,
            300,
            vec![EnergyType::Water, EnergyType::Water, EnergyType::Water],
        );
        if evolve_now {
            state.set_board(
                vec![PlayedCard::from_id(CardId::B3a059Dunsparce)
                    .with_energy(vec![EnergyType::Colorless, EnergyType::Colorless])],
                vec![defender],
            );
            state.hands[0] = vec![get_card_by_enum(CardId::B3a060Dudunsparce)];
        } else {
            state.set_board(
                vec![PlayedCard::from_id(CardId::B3a060Dudunsparce)
                    .with_energy(vec![EnergyType::Colorless, EnergyType::Colorless])],
                vec![defender],
            );
        }
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        if evolve_now {
            game.apply_action(&Action {
                actor: 0,
                action: SimpleAction::Evolve {
                    evolution: get_card_by_enum(CardId::B3a060Dudunsparce),
                    in_play_idx: 0,
                    from_deck: false,
                },
                is_stack: false,
            });
        }
        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B3a060Dudunsparce, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).attached_energy.len(),
            expected_energy,
            "evolved this turn = {evolve_now}",
        );
    }
}

/// Delcatty's "Energy Blender": 50 damage, "You may move any amount of Energy from your Pokémon
/// in play to your other Pokémon in any way you like."
#[test]
fn test_delcatty_energy_blender_can_move_energy() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B4135Delcatty)
                .with_energy(vec![EnergyType::Colorless, EnergyType::Water]),
            PlayedCard::from_id(CardId::A1053Squirtle),
        ],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B4135Delcatty, 0),
        is_stack: false,
    });

    let (_, choices) = game.get_state_clone().generate_possible_actions();
    assert!(
        choices
            .iter()
            .any(|a| matches!(a.action, SimpleAction::MoveEnergy { .. })),
        "moving Energy is on offer: {choices:?}",
    );
    assert!(
        choices
            .iter()
            .any(|a| matches!(a.action, SimpleAction::Noop)),
        "and so is declining: {choices:?}",
    );
}

/// Smeargle's "Portrait": "Once during your turn, if this Pokémon is in the Active Spot, you may
/// look at a random Supporter card from your opponent's hand. Use the effect of that card as the
/// effect of this Ability."
#[test]
fn test_smeargle_portrait_borrows_a_supporter() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2130Smeargle)],
        // Giovanni gives +10 damage this turn; Professor's Research draws 2.
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.hands[0].clear();
    state.hands[1] = vec![
        get_card_by_enum(CardId::PA001Potion), // Item: not a Supporter
        get_card_by_enum(CardId::PA007ProfessorsResearch),
    ];
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1001Bulbasaur); 4];
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
        state.hands[0].len(),
        2,
        "Professor's Research draws 2 for Smeargle's owner",
    );
    assert_eq!(state.hands[1].len(), 2, "the opponent keeps their cards");
}

/// Portrait runs the effect without the card being played, so the borrowed Supporter's own
/// "can I be played?" check is skipped. Cyrus needs a damaged Benched Pokemon on the other side;
/// without one it used to push an empty list of choices and the game had no legal move.
#[test]
fn test_smeargle_portrait_skips_a_supporter_that_cannot_be_played() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2130Smeargle)],
        // The opponent's Bench is empty, so Cyrus has nothing to switch in.
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
    );
    state.hands[0].clear();
    state.hands[1] = vec![get_card_by_enum(CardId::A2150Cyrus)];
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1001Bulbasaur); 4];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::UseAbility { in_play_idx: 0 },
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert!(
        state
            .move_generation_stack
            .iter()
            .all(|(_, choices)| !choices.is_empty()),
        "選択肢が 0 件のまま積まれている: {:?}",
        state.move_generation_stack,
    );
    let (_, actions) = state.generate_possible_actions();
    assert!(!actions.is_empty(), "打つ手がなくなっている");
}
