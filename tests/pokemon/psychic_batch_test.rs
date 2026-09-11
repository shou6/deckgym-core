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

/// Grumpig's "Swaying Dance": 40 damage, "If your opponent has exactly 2, 4, or 6 cards in their
/// hand, this attack does 40 more damage." The mirror of Ludicolo's Rhythmic Steps, which counts
/// your own hand.
#[test]
fn test_grumpig_swaying_dance_counts_the_opponents_hand() {
    for (hand_size, expected_hp) in [(4usize, 220u32), (5, 260)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B2b032Grumpig).with_energy(vec![EnergyType::Psychic])],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![],
            )],
        );
        state.hands[1] = vec![get_card_by_enum(CardId::A1033Charmander); hand_size];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2b032Grumpig, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "opponent holding {hand_size} cards",
        );
    }
}

/// Chimecho's "Extrasensory": 40 damage, "If you have the same number of cards in your hand as
/// your opponent, this attack does 40 more damage."
#[test]
fn test_chimecho_extrasensory_compares_hands() {
    for (own, theirs, expected_hp) in [(3usize, 3usize, 220u32), (3, 4, 260)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4073Chimecho)
                .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless])],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![],
            )],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A1033Charmander); own];
        state.hands[1] = vec![get_card_by_enum(CardId::A1033Charmander); theirs];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4073Chimecho, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "{own} cards vs {theirs}",
        );
    }
}

/// Mr. Mime's "Synchro Dance": 40 damage, "If this Pokémon and your opponent's Active Pokémon
/// have the same amount of Energy attached, this attack does 40 more damage."
#[test]
fn test_mr_mime_synchro_dance_compares_energy_counts() {
    for (defender_energy, expected_hp) in [(2usize, 220u32), (1, 260)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4069MrMime)
                .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless])],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![EnergyType::Fire; defender_energy],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4069MrMime, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "defender holding {defender_energy} Energy against 2",
        );
    }
}

/// Enamorus's "Smitten Strike": 60 damage, "If this Pokémon and your opponent's Active Pokémon
/// have 1 or more of the same type of Energy attached, this attack does 60 more damage."
#[test]
fn test_enamorus_smitten_strike_needs_a_shared_type() {
    for (defender_energy, expected_hp) in [
        (EnergyType::Psychic, 180u32), // shared: 60 + 60
        (EnergyType::Fire, 240),       // nothing in common
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B3b033Enamorus).with_energy(vec![
                    EnergyType::Psychic,
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                ]),
            ],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![defender_energy],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B3b033Enamorus, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "defender holding {defender_energy:?}",
        );
    }
}

/// Flutter Mane's "Hexing Flight": 90 damage, "If this Pokémon didn't move from the Bench to the
/// Active Spot this turn, this attack does nothing."
#[test]
fn test_flutter_mane_hexing_flight_needs_a_fresh_retreat() {
    for (retreat_first, expected_hp) in [(true, 210u32), (false, 300)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::A1033Charmander).with_energy(vec![EnergyType::Fire]),
                PlayedCard::from_id(CardId::B3b035FlutterMane)
                    .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless]),
            ],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        if !retreat_first {
            // Start with Flutter Mane already Active instead of retreating into the spot.
            state.set_board(
                vec![PlayedCard::from_id(CardId::B3b035FlutterMane)
                    .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless])],
                vec![played_card_with_base_hp(
                    CardId::A1033Charmander,
                    300,
                    vec![],
                )],
            );
            state.current_player = 0;
            state.turn_count = 5;
        }
        game.set_state(state);

        if retreat_first {
            game.apply_action(&Action {
                actor: 0,
                action: SimpleAction::Retreat(1),
                is_stack: false,
            });
        }
        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B3b035FlutterMane, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "moved from the Bench this turn = {retreat_first}",
        );
    }
}

/// Team Rocket's Wobbuffet's "Rocket Frenzy": "Reveal the top 6 cards of your deck. This attack
/// does 30 damage for each Pokémon you find there that has "Team Rocket" in its name."
#[test]
fn test_team_rockets_wobbuffet_rocket_frenzy_counts_the_gang() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::PB091TeamRocketsWobbuffet)
            .with_energy(vec![EnergyType::Colorless, EnergyType::Colorless])],
        vec![played_card_with_base_hp(
            CardId::A1033Charmander,
            300,
            vec![],
        )],
    );
    state.decks[0].cards = vec![
        get_card_by_enum(CardId::B4a041TeamRocketsMuk),
        get_card_by_enum(CardId::A1033Charmander),
        get_card_by_enum(CardId::B4a013TeamRocketsLapras),
        get_card_by_enum(CardId::A1033Charmander),
        get_card_by_enum(CardId::A1033Charmander),
        get_card_by_enum(CardId::A1033Charmander),
        // The 7th card is out of reach even though it belongs to the gang.
        get_card_by_enum(CardId::B4a020TeamRocketsElectrode),
    ];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::PB091TeamRocketsWobbuffet, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        240,
        "30 for each of the 2 Team Rocket Pokemon in the top 6, and nothing else",
    );
    assert_eq!(
        state.decks[0].cards.len(),
        7,
        "the cards go back into the deck"
    );
}

/// Mew's "Psy Report": "Your opponent reveals their hand." Nothing is hidden in this engine, so
/// the reveal is only the damage.
#[test]
fn test_mew_psy_report_is_just_damage() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A1283Mew).with_energy(vec![EnergyType::Psychic])],
        vec![played_card_with_base_hp(
            CardId::A1033Charmander,
            300,
            vec![],
        )],
    );
    state.hands[1] = vec![get_card_by_enum(CardId::A1033Charmander); 3];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A1283Mew, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 280, "20 damage");
    assert_eq!(state.hands[1].len(), 3, "the hand is untouched");
}

/// Musharna's "Dream Dance": 60 damage, "Both Active Pokémon are now Asleep."
#[test]
fn test_musharna_dream_dance_puts_both_to_sleep() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A4091Musharna)
            .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless])],
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
        action: attack_action(CardId::A4091Musharna, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 240, "60 damage");
    assert!(state.get_active(1).is_asleep(), "the defender falls asleep");
    assert!(state.get_active(0).is_asleep(), "and so does Musharna");
}

/// Mimikyu's "Shadow Hit": 60 damage, "This attack also does 20 damage to 1 of your Pokémon."
#[test]
fn test_mimikyu_shadow_hit_costs_you_20() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A3083Mimikyu)
                .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless]),
            played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
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
        action: attack_action(CardId::A3083Mimikyu, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        240,
        "60 to the defender"
    );
    // Mimikyu has 70 HP printed and the Benched Charmander was given 300.
    let own_hp: Vec<u32> = state
        .enumerate_in_play_pokemon(0)
        .map(|(_, p)| p.get_remaining_hp())
        .collect();
    assert_eq!(
        own_hp.iter().filter(|hp| **hp == 50 || **hp == 280).count(),
        1,
        "20 lands on one of your own Pokemon: {own_hp:?}",
    );
}

/// Diancie's "Diamond Storm": 50 damage, "Heal 20 damage from each of your [P] Pokémon."
#[test]
fn test_diancie_diamond_storm_heals_the_psychics() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B3067Diancie)
                .with_energy(vec![EnergyType::Psychic, EnergyType::Psychic])
                .with_damage(30),
            PlayedCard::from_id(CardId::A3083Mimikyu).with_damage(30),
            // A non-[P] Pokemon is not healed.
            PlayedCard::from_id(CardId::A1033Charmander).with_damage(30),
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
        action: attack_action(CardId::B3067Diancie, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 250, "50 damage");
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        80,
        "Diancie heals 20 of its 30 damage (90 HP printed)",
    );
    assert_eq!(
        state.in_play_pokemon[0][1]
            .as_ref()
            .expect("Mimikyu is there")
            .get_remaining_hp(),
        60,
        "and so does the Benched [P] Pokemon (70 HP printed)",
    );
    assert_eq!(
        state.in_play_pokemon[0][2]
            .as_ref()
            .expect("Charmander is there")
            .get_remaining_hp(),
        30,
        "the [R] Pokemon keeps its damage (60 HP printed)",
    );
}

/// Espathra's "Lumina Crash": 50 damage, "During your next turn, the Defending Pokémon takes +50
/// damage from attacks."
#[test]
fn test_espathra_lumina_crash_softens_the_defender() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B3a022Espathra)
            .with_energy(vec![EnergyType::Psychic, EnergyType::Psychic])],
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
        action: attack_action(CardId::B3a022Espathra, 0),
        is_stack: false,
    });

    assert!(
        game.get_state_clone()
            .get_active(1)
            .get_active_effects()
            .iter()
            .any(|e| matches!(
                e,
                deckgym::effects::CardEffect::IncreasedVulnerability { amount: 50 }
            )),
        "the defender is left more vulnerable",
    );
}

/// Latios's "Fantastical Floating": "If you have Latias in play, this Pokémon has no Retreat
/// Cost." Its printed cost is 2.
#[test]
fn test_latios_fantastical_floating_needs_latias() {
    for (bencher, expected) in [
        (CardId::A4a036Latias, true),
        (CardId::A1033Charmander, false),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::A4a037Latios),
                PlayedCard::from_id(bencher),
            ],
            vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
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

/// Galarian Cursola's "Perish Body": "If this Pokémon is in the Active Spot and is Knocked Out by
/// damage from an attack from your opponent's Pokémon, flip a coin. If heads, the Attacking
/// Pokémon is Knocked Out."
#[test]
fn test_galarian_cursola_perish_body_can_take_the_attacker_along() {
    let mut taken = 0;
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
                PlayedCard::from_id(CardId::A1001Bulbasaur),
            ],
            vec![
                played_card_with_base_hp(CardId::A4a035GalarianCursola, 70, vec![]),
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
        if game.get_state_clone().in_play_pokemon[0][0]
            .as_ref()
            .is_none_or(|p| p.get_name() != "Poliwrath")
        {
            taken += 1;
        }
    }
    assert!(
        taken > 0 && taken < seed_count,
        "the attacker goes down on heads only (taken {taken}/{seed_count})",
    );
}

/// Scream Tail's "Shooing Shout": "Flip 2 coins. If both of them are heads, discard your
/// opponent's Active Pokémon."
#[test]
fn test_scream_tail_shooing_shout_needs_two_heads() {
    let mut discarded = 0;
    let seed_count = 40u64;
    for seed in 0..seed_count {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B3a025ScreamTail)
                .with_energy(vec![EnergyType::Psychic, EnergyType::Psychic])],
            vec![
                played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
                PlayedCard::from_id(CardId::A1033Charmander),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B3a025ScreamTail, 0),
            is_stack: false,
        });
        if game.get_state_clone().in_play_pokemon[1][0].is_none() {
            discarded += 1;
        }
    }
    // Two heads is a quarter of the time, so it should be rarer than a single coin.
    assert!(
        discarded > 0 && discarded < seed_count / 2,
        "both coins must come up heads (discarded {discarded}/{seed_count})",
    );
}

/// Mime Jr.'s "Mime-y Shuffle": "Shuffle your hand into your deck. Draw a card for each card in
/// your opponent's hand."
#[test]
fn test_mime_jr_shuffle_matches_the_opponents_hand() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B4068MimeJr)],
        vec![played_card_with_base_hp(
            CardId::A1033Charmander,
            300,
            vec![],
        )],
    );
    state.hands[0] = vec![get_card_by_enum(CardId::A1033Charmander); 2];
    state.hands[1] = vec![get_card_by_enum(CardId::A1033Charmander); 4];
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1001Bulbasaur); 6];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B4068MimeJr, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.hands[0].len(),
        4,
        "the new hand matches the opponent's"
    );
    assert_eq!(
        state.decks[0].cards.len(),
        4,
        "2 went back in and 4 came out of a 6 card deck",
    );
}

/// Hoopa's "Mischievous Ring": 20 damage, "Before doing damage, shuffle all Pokémon Tools from
/// each of your opponent's Pokémon into their deck."
#[test]
fn test_hoopa_mischievous_ring_strips_every_tool() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B4077Hoopa).with_energy(vec![EnergyType::Colorless])],
        vec![
            played_card_with_base_hp(CardId::A1033Charmander, 300, vec![])
                .with_tool(get_card_by_enum(CardId::A2148RockyHelmet)),
            PlayedCard::from_id(CardId::A1033Charmander)
                .with_tool(get_card_by_enum(CardId::A3146PoisonBarb)),
        ],
    );
    state.decks[1].cards = vec![];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B4077Hoopa, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert!(
        state.get_active(1).attached_tool.is_none(),
        "the Active Pokemon's Tool is gone",
    );
    assert!(
        state.in_play_pokemon[1][1]
            .as_ref()
            .expect("the Benched Pokemon is there")
            .attached_tool
            .is_none(),
        "and the Benched one's too",
    );
    assert_eq!(state.decks[1].cards.len(), 2, "both Tools go into the deck");
}

/// Uxie's "Mind Boost": 20 damage, "Take a [P] Energy from your Energy Zone and attach it to
/// Mesprit or Azelf." Only those two are offered.
#[test]
fn test_uxie_mind_boost_feeds_its_partners() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A2075Uxie).with_energy(vec![EnergyType::Psychic]),
            PlayedCard::from_id(CardId::A2076Mesprit),
            PlayedCard::from_id(CardId::A1033Charmander),
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
        action: attack_action(CardId::A2075Uxie, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert_eq!(
        state.in_play_pokemon[0][1]
            .as_ref()
            .expect("Mesprit is there")
            .attached_energy,
        vec![EnergyType::Psychic],
        "Mesprit takes the Energy",
    );
    assert!(
        state.in_play_pokemon[0][2]
            .as_ref()
            .expect("Charmander is there")
            .attached_energy
            .is_empty(),
        "and nobody else is offered it",
    );
}

/// Mesprit's "Supreme Blast": 160 damage, "You can use this attack only if you have Uxie and
/// Azelf on your Bench. Discard all Energy from this Pokémon."
#[test]
fn test_mesprit_supreme_blast_needs_the_trio() {
    for (partners, expected) in [
        (vec![CardId::A2075Uxie, CardId::A2077Azelf], true),
        (vec![CardId::A2075Uxie, CardId::A1033Charmander], false),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::A2076Mesprit).with_energy(vec![
                    EnergyType::Psychic,
                    EnergyType::Psychic,
                    EnergyType::Psychic,
                ]),
                PlayedCard::from_id(partners[0]),
                PlayedCard::from_id(partners[1]),
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

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let can_attack = actions
            .iter()
            .any(|a| matches!(&a.action, SimpleAction::Attack(attack) if attack.title == "Supreme Blast"));
        assert_eq!(can_attack, expected, "Bench {partners:?}");
    }
}

/// Wobbuffet's "Reply Strongly": 30 damage, "If this Pokémon was damaged by an attack during your
/// opponent's last turn while it was in the Active Spot, this attack does 50 more damage."
#[test]
fn test_wobbuffet_reply_strongly_answers_a_hit() {
    for (hit_last_turn, expected_hp) in [(true, 220u32), (false, 270)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A4086Wobbuffet)
                .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless])],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        state.set_active_damaged_last_turn(0, hit_last_turn);
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A4086Wobbuffet, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "damaged during the opponent's last turn = {hit_last_turn}",
        );
    }
}

/// Clefairy's "Mini-Metronome": "Flip a coin. If heads, choose 1 of your opponent's Active
/// Pokémon's attacks and use it as this attack."
#[test]
fn test_clefairy_mini_metronome_borrows_on_heads() {
    let mut copied = 0;
    let seed_count = 24u64;
    for seed in 0..seed_count {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4064Clefairy).with_energy(vec![
                EnergyType::Colorless,
                EnergyType::Colorless,
                EnergyType::Colorless,
            ])],
            // Poliwrath's Mega Punch does 80; Clefairy's own attack does nothing.
            vec![played_card_with_base_hp(
                CardId::A1061Poliwrath,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4064Clefairy, 0),
            is_stack: false,
        });
        resolve_stacked_actions(&mut game);
        if game.get_state_clone().get_active(1).get_remaining_hp() < 300 {
            copied += 1;
        }
    }
    assert!(
        copied > 0 && copied < seed_count,
        "the borrowed attack follows a coin flip (copied {copied}/{seed_count})",
    );
}

/// Polteageist's "Refreshing Tea": "Once during your turn, when you play this Pokémon from your
/// hand to evolve 1 of your Pokémon, you may have your opponent shuffle their hand into their
/// deck. For each remaining point that your opponent needs to win, they draw a card."
#[test]
fn test_polteageist_refreshing_tea_resets_the_opponents_hand() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2074Sinistea)],
        vec![played_card_with_base_hp(
            CardId::A1033Charmander,
            300,
            vec![],
        )],
    );
    state.hands[0] = vec![get_card_by_enum(CardId::B2075Polteageist)];
    state.hands[1] = vec![get_card_by_enum(CardId::A1033Charmander); 5];
    state.decks[1].cards = vec![get_card_by_enum(CardId::A1001Bulbasaur); 6];
    state.points[1] = 1; // 2 points still needed, so 2 cards come back
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Evolve {
            evolution: get_card_by_enum(CardId::B2075Polteageist),
            in_play_idx: 0,
            from_deck: false,
        },
        is_stack: false,
    });
    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    let choice = choices
        .into_iter()
        .find(|a| !matches!(a.action, SimpleAction::Noop))
        .expect("Refreshing Tea is on offer after evolving");
    game.apply_action(&Action {
        actor,
        action: choice.action,
        is_stack: true,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.hands[1].len(),
        2,
        "the opponent draws 1 card per point they still need",
    );
    assert_eq!(
        state.decks[1].cards.len(),
        9,
        "5 went in and 2 came out of a 6 card deck",
    );
}

/// Gothitelle's "Stellar Cradle": 70 damage, "During your opponent's next turn, if they attach
/// Energy from their Energy Zone to the Defending Pokémon, that Pokémon will be Asleep."
#[test]
fn test_gothitelle_stellar_cradle_punishes_an_attach() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B1114Gothitelle)
            .with_energy(vec![EnergyType::Psychic, EnergyType::Psychic])],
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
        action: attack_action(CardId::B1114Gothitelle, 0),
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 1,
        action: SimpleAction::Attach {
            attachments: vec![(1, EnergyType::Fire, 0)],
            is_turn_energy: true,
        },
        is_stack: false,
    });

    assert!(
        game.get_state_clone().get_active(1).is_asleep(),
        "attaching to the cradled Pokemon puts it to sleep",
    );
}

/// Unown's "CHECK": "Once during your turn, you may choose either player. Look at the top card of
/// that player's deck." Nothing is hidden in this engine, so it changes nothing.
#[test]
fn test_unown_check_is_information_only() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A2a034Unown)],
        vec![played_card_with_base_hp(
            CardId::A1033Charmander,
            300,
            vec![],
        )],
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

/// Unown's "GUARD" and "POWER" only work alongside a different Unown Ability: GUARD takes 10 off
/// what your Pokémon take, POWER adds 10 to what they deal.
#[test]
fn test_unown_guard_and_power_need_each_other() {
    for (partner, expected_hp) in [
        (CardId::A4085Unown, 230u32), // POWER is a different Ability: GUARD works, 80 - 10
        (CardId::A4084Unown, 220),    // another GUARD is not enough, so the full 80 lands
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
            ],
            vec![
                played_card_with_base_hp(CardId::A4084Unown, 300, vec![]),
                PlayedCard::from_id(partner),
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
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "GUARD alongside {partner:?}",
        );
    }
}

/// Mew's "Miraculous Memory": "1 attack from among the Pokémon in your opponent's hand and deck
/// is chosen at random, and you use the chosen attack as this attack."
#[test]
fn test_mew_miraculous_memory_borrows_from_hand_and_deck() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2b030Mew)
            .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless])],
        vec![played_card_with_base_hp(
            CardId::A1033Charmander,
            300,
            vec![],
        )],
    );
    // Every attack in reach does a fixed 80, so the random pick is still checkable.
    state.hands[1] = vec![get_card_by_enum(CardId::A1061Poliwrath)];
    state.decks[1].cards = vec![get_card_by_enum(CardId::A1061Poliwrath); 2];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2b030Mew, 0),
        is_stack: false,
    });
    resolve_stacked_actions(&mut game);

    assert_eq!(
        game.get_state_clone().get_active(1).get_remaining_hp(),
        220,
        "Mega Punch's 80 is what comes back",
    );
}
