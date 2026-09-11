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

/// Machop's "Shatter" and Conkeldurr's "Bedrock Breaker" both read "Discard a
/// Stadium in play." Whoever put it there, it goes.
#[test]
fn test_shatter_discards_the_stadium_whoever_owns_it() {
    for owner in [0usize, 1] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B2079Machop).with_energy(vec![EnergyType::Colorless])],
            vec![played_card_with_base_hp(
                CardId::A1001Bulbasaur,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        state.active_stadium = Some(get_card_by_enum(CardId::B3153FragrantForest));
        state.active_stadium_owner = Some(owner);
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2079Machop, 0),
            is_stack: false,
        });

        let state = game.get_state_clone();
        assert!(
            state.active_stadium.is_none(),
            "the Stadium should be gone (owner {owner})"
        );
        assert_eq!(state.get_active(1).get_remaining_hp(), 290, "10 damage");
    }
}

/// Nothing in play means nothing to discard; the attack still lands.
#[test]
fn test_shatter_is_fine_with_no_stadium() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2079Machop).with_energy(vec![EnergyType::Colorless])],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![],
        )],
    );
    state.current_player = 0;
    state.turn_count = 5;
    state.active_stadium = None;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2079Machop, 0),
        is_stack: false,
    });
    assert_eq!(game.get_state_clone().get_active(1).get_remaining_hp(), 290);
}

/// Dugtrio's "Cliff Crumbler": 40 damage, "Discard the top card of your deck. If
/// that card is a [F] Pokémon, this attack does 60 more damage." Same shape as
/// Pachirisu's Item check, but on a Pokemon type.
#[test]
fn test_dugtrio_cliff_crumbler_reads_the_top_card() {
    for (top, expected_hp) in [
        (CardId::A1155Hitmonchan, 200u32), // a [F] Pokemon
        (CardId::A1001Bulbasaur, 260),     // [G], so no bonus
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A4a041Dugtrio)
                .with_energy(vec![EnergyType::Fighting])],
            vec![played_card_with_base_hp(CardId::A1033Charmander, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        let deck_before = state.decks[0].cards.len();
        state.decks[0].cards.insert(0, get_card_by_enum(top));
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A4a041Dugtrio, 0),
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

/// Tyrantrum's "Tyrannical Fang": 100 damage, "+80 if you have fewer Pokémon in
/// play than your opponent."
#[test]
fn test_tyrantrum_rewards_being_outnumbered() {
    for (own_bench, expected_hp) in [(1usize, 200u32), (0, 120)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        let mut own = vec![
            PlayedCard::from_id(CardId::B2090Tyrantrum).with_energy(vec![
                EnergyType::Fighting,
                EnergyType::Fighting,
                EnergyType::Fighting,
            ]),
        ];
        for _ in 0..own_bench {
            own.push(PlayedCard::from_id(CardId::A1033Charmander));
        }
        state.set_board(
            own,
            vec![
                played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
                PlayedCard::from_id(CardId::A1001Bulbasaur),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2090Tyrantrum, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "own board of {} vs the opponent's 2",
            own_bench + 1
        );
    }
}

/// Marowak's "Punish": 50 damage, "+70 if your opponent's Active Pokémon has
/// \u{201c}Team Rocket\u{201d} in its name."
#[test]
fn test_marowak_punish_looks_for_team_rocket_in_the_name() {
    // Team Rocket's Electrode is Weak to [F], so its row also carries the +20:
    // 50 + 70 + 20 = 140.
    for (defender, expected_hp) in [
        (CardId::B4a020TeamRocketsElectrode, 160u32),
        (CardId::A1033Charmander, 250),
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B4a036Marowak)
                .with_energy(vec![EnergyType::Fighting, EnergyType::Colorless])],
            vec![played_card_with_base_hp(defender, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B4a036Marowak, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
        );
    }
}

/// Groudon's "Gaia Blast": 130 damage, "Discard 2 random Energy from among the
/// Energy attached to all of your Pokémon." Its own side only.
#[test]
fn test_groudon_gaia_blast_pays_from_its_own_side() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B2b035Groudon).with_energy(vec![
                EnergyType::Fighting,
                EnergyType::Fighting,
                EnergyType::Colorless,
                EnergyType::Colorless,
            ]),
            PlayedCard::from_id(CardId::A1033Charmander).with_energy(vec![EnergyType::Fire]),
        ],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![EnergyType::Grass, EnergyType::Grass],
        )],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2b035Groudon, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(state.get_active(1).get_remaining_hp(), 170, "130 damage");
    let own_energy: usize = state.in_play_pokemon[0]
        .iter()
        .flatten()
        .map(|p| p.attached_energy.len())
        .sum();
    assert_eq!(own_energy, 3, "two of its own five Energy are discarded");
    assert_eq!(
        state.get_active(1).attached_energy.len(),
        2,
        "the opponent's Energy is untouched"
    );
}

/// Meloetta's "Inspiring Dance": 30 damage, "During your next turn, attacks used
/// by your [F] Pokémon do +30 damage to your opponent's Active Pokémon."
#[test]
fn test_meloetta_inspiring_dance_boosts_fighting_next_turn() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B3089Meloetta)
                .with_energy(vec![EnergyType::Fighting, EnergyType::Colorless]),
            // Hitmonchan is [F]; its Jab does 30, so next turn it should do 60.
            PlayedCard::from_id(CardId::A1155Hitmonchan).with_energy(vec![EnergyType::Fighting]),
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
        action: attack_action(CardId::B3089Meloetta, 0),
        is_stack: false,
    });
    let after_dance = game.get_state_clone().get_active(1).get_remaining_hp();
    assert_eq!(after_dance, 270, "Inspiring Dance itself does 30");

    // Swap Hitmonchan in and attack on the following turn.
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A1155Hitmonchan).with_energy(vec![EnergyType::Fighting])],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            270,
            vec![],
        )],
    );
    game.set_state(state);
    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A1155Hitmonchan, 0),
        is_stack: false,
    });

    assert_eq!(
        game.get_state_clone().get_active(1).get_remaining_hp(),
        210,
        "Jab's 30 plus Inspiring Dance's 30"
    );
}

/// Sandy Shocks's "Pull In and Pound": "Switch in 1 of your opponent's Benched
/// Pokémon to the Active Spot. If you do, this attack does 50 damage to the new
/// Active Pokémon." The damage follows the Pokemon that was dragged out.
#[test]
fn test_sandy_shocks_damages_whoever_it_drags_out() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B3a035SandyShocks).with_energy(vec![
                EnergyType::Fighting,
                EnergyType::Fighting,
                EnergyType::Colorless,
            ]),
        ],
        vec![
            played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
            played_card_with_base_hp(CardId::A1120Gastly, 300, vec![]),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B3a035SandyShocks, 0),
        is_stack: false,
    });

    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    assert_eq!(actor, 0, "the attacker chooses who comes out");
    assert_eq!(choices.len(), 1, "only Gastly is on the bench: {choices:?}");
    game.apply_action(&choices[0].clone());
    // The damage is the next step on the stack.
    game.play_until_stable();

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_name(),
        "Gastly",
        "Gastly is dragged into the Active Spot"
    );
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        250,
        "and takes the 50"
    );
}

/// Nothing on the bench means nothing to drag, and no damage either.
#[test]
fn test_sandy_shocks_does_nothing_without_a_bench() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B3a035SandyShocks).with_energy(vec![
                EnergyType::Fighting,
                EnergyType::Fighting,
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
        action: attack_action(CardId::B3a035SandyShocks, 0),
        is_stack: false,
    });
    game.play_until_stable();

    assert_eq!(
        game.get_state_clone().get_active(1).get_remaining_hp(),
        300,
        "no bench, no switch, no damage"
    );
}

/// Golurk's "Heavy Rocket": 60 damage, "Reveal the top 3 cards of your deck. This
/// attack does 60 damage for each Pokémon with a Retreat Cost of 3 or more you
/// find there. Shuffle the revealed cards back into your deck."
#[test]
fn test_golurk_heavy_rocket_counts_heavy_pokemon_on_top() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B1136Golurk)
            .with_energy(vec![EnergyType::Fighting, EnergyType::Fighting])],
        vec![played_card_with_base_hp(
            CardId::A1033Charmander,
            300,
            vec![],
        )],
    );
    state.current_player = 0;
    state.turn_count = 5;
    let deck_before = state.decks[0].cards.len();
    // Wimpod's Retreat Cost is 3; two of them on top is 60 + 60 + 60.
    state.decks[0]
        .cards
        .insert(0, get_card_by_enum(CardId::A1001Bulbasaur));
    state.decks[0]
        .cards
        .insert(0, get_card_by_enum(CardId::A3021Wimpod));
    state.decks[0]
        .cards
        .insert(0, get_card_by_enum(CardId::A3021Wimpod));
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B1136Golurk, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        120,
        "60 base plus 60 for each of the two heavy Pokemon"
    );
    assert_eq!(
        state.decks[0].cards.len(),
        deck_before + 3,
        "the revealed cards go back into the deck"
    );
}

/// Archeops's "Wild Spin": "This attack does 20 damage to each of your opponent's
/// Pokémon. During your next turn, this Pokémon's Wild Spin attack does +20
/// damage to each of your opponent's Pokémon." The boost compounds on itself.
#[test]
fn test_archeops_wild_spin_boosts_its_own_next_use() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B1134Archeops).with_energy(vec![EnergyType::Fighting])],
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
        action: attack_action(CardId::B1134Archeops, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        280,
        "20 to the Active"
    );
    assert_eq!(
        state.in_play_pokemon[1][1]
            .as_ref()
            .expect("opponent bench")
            .get_remaining_hp(),
        280,
        "and 20 to the Bench"
    );

    // Second use, still within the boost.
    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B1134Archeops, 0),
        is_stack: false,
    });
    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        240,
        "the second Wild Spin does 40"
    );
}

/// Quagsire's "Amnesia": 60 damage, "1 of your opponent's Active Pokémon's attacks
/// is chosen at random. During your opponent's next turn, that Pokémon can't use
/// the chosen attack."
#[test]
fn test_quagsire_amnesia_locks_one_of_the_defenders_attacks() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B3b037Quagsire).with_energy(vec![
                EnergyType::Fighting,
                EnergyType::Fighting,
                EnergyType::Colorless,
            ]),
        ],
        vec![
            // Bulbasaur has a single attack, so the random pick is deterministic.
            PlayedCard::from_id(CardId::A1001Bulbasaur).with_energy(vec![EnergyType::Grass]),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B3b037Quagsire, 0),
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 0,
        action: deckgym::actions::SimpleAction::EndTurn,
        is_stack: false,
    });

    let (_, actions) = game.get_state_clone().generate_possible_actions();
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a.action, deckgym::actions::SimpleAction::Attack(_))),
        "Vine Whip was the only attack and it is locked: {actions:?}"
    );
}

/// Armaldo's "Abyssal Drop": "Discard all Energy from this Pokémon. Choose a spot
/// from among your opponent's Active Spot and Bench. At the end of your opponent's
/// next turn, Knock Out the Pokémon in the spot you chose." The knockout is
/// modelled as delayed damage large enough to finish anything.
#[test]
fn test_armaldo_abyssal_drop_pays_its_energy_and_marks_a_spot() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B4082Armaldo).with_energy(vec![
            EnergyType::Fighting,
            EnergyType::Colorless,
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
        action: attack_action(CardId::B4082Armaldo, 0),
        is_stack: false,
    });

    assert!(
        game.get_state_clone()
            .get_active(0)
            .attached_energy
            .is_empty(),
        "Armaldo pays every Energy it had"
    );

    let (actor, choices) = game.get_state_clone().generate_possible_actions();
    assert_eq!(actor, 0, "the attacker marks the spot");
    assert_eq!(
        choices.len(),
        2,
        "the Active Spot and the Bench are both choosable: {choices:?}"
    );
    game.apply_action(&choices[0].clone());

    // End both turns; the marked Pokemon is knocked out at the end of the
    // opponent's next turn.
    game.apply_action(&Action {
        actor: 0,
        action: deckgym::actions::SimpleAction::EndTurn,
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 1,
        action: deckgym::actions::SimpleAction::EndTurn,
        is_stack: false,
    });
    game.play_until_stable();

    let state = game.get_state_clone();
    let marked_alive = state.in_play_pokemon[1]
        .iter()
        .flatten()
        .any(|p| p.get_name() == "Charmander" && p.get_remaining_hp() == 300);
    assert!(
        !marked_alive,
        "the marked Charmander should have been knocked out"
    );
}

/// Falinks's "Coordinated Unit": "If you have another Falinks in play, this Pokémon's
/// attacks do +20 damage to your opponent's Active Pokémon, and this Pokémon takes -20 damage
/// from attacks from your opponent's Pokémon."
#[test]
fn test_falinks_formation_needs_a_second_falinks() {
    for (ally, expected_defender_hp) in [
        (CardId::B2092Falinks, 260u32), // 20 + 20
        (CardId::A1053Squirtle, 280),   // 20
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B2092Falinks).with_energy(vec![EnergyType::Fighting]),
                PlayedCard::from_id(ally),
            ],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2092Falinks, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_defender_hp,
            "ally {ally:?}",
        );
    }
}

/// The other half of the formation: the shield only stands with a second Falinks in play.
#[test]
fn test_falinks_formation_softens_incoming_damage() {
    for (ally, expected_falinks_hp) in [
        (CardId::B2092Falinks, 240u32), // 80 - 20
        (CardId::A1053Squirtle, 220),   // 80
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
                played_card_with_base_hp(CardId::B2092Falinks, 300, vec![]),
                PlayedCard::from_id(ally),
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
            expected_falinks_hp,
            "Falinks with ally {ally:?}",
        );
    }
}

/// Hippowdon's "Crashing Fangs": 100 damage, "Flip a coin. If tails, during your next turn, this
/// Pokémon can't attack." Same shape as Origin Forme Dialga's Time Mash.
#[test]
fn test_hippowdon_crashing_fangs_locks_itself_on_tails() {
    let mut locked = 0;
    let seed_count = 24u64;
    for seed in 0..seed_count {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B1129Hippowdon).with_energy(vec![
                    EnergyType::Fighting,
                    EnergyType::Fighting,
                    EnergyType::Colorless,
                ]),
            ],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B1129Hippowdon, 0),
            is_stack: false,
        });

        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(1).get_remaining_hp(),
            200,
            "100 damage whichever way the coin lands (seed {seed})",
        );
        if state
            .get_active(0)
            .get_active_effects()
            .iter()
            .any(|e| matches!(e, deckgym::effects::CardEffect::CannotAttack))
        {
            locked += 1;
        }
    }
    assert!(
        locked > 0 && locked < seed_count,
        "the lock should follow a coin flip, not always or never (locked {locked}/{seed_count})",
    );
}

/// Ting-Lu's "Arrogant Impact": 130 damage, "If this Pokémon's remaining HP is 60 or less, this
/// attack does nothing."
#[test]
fn test_ting_lu_arrogant_impact_needs_a_healthy_ting_lu() {
    for (damage_on_self, expected_hp) in [(0u32, 170u32), (60, 300)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B2a062TingLu)
                .with_energy(vec![
                    EnergyType::Fighting,
                    EnergyType::Fighting,
                    EnergyType::Fighting,
                ])
                .with_damage(damage_on_self)],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2a062TingLu, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "Ting-Lu carrying {damage_on_self} damage (120 HP printed)",
        );
    }
}
