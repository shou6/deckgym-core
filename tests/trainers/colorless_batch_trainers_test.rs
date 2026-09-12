use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::get_initialized_game,
    Game, State,
};

fn play(game: &mut Game, trainer: CardId) {
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Play {
            trainer_card: get_card_by_enum(trainer).as_trainer(),
        },
        is_stack: false,
    });
}

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

fn board_with(
    trainer: CardId,
    own: Vec<PlayedCard>,
    theirs: Vec<PlayedCard>,
) -> (Game<'static>, State) {
    let game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(own, theirs);
    state.hands[0] = vec![get_card_by_enum(trainer)];
    state.current_player = 0;
    state.turn_count = 5;
    (game, state)
}

/// Pokémon Flute: "Put a Basic Pokémon from your opponent's discard pile onto their Bench."
#[test]
fn test_pokemon_flute_revives_onto_the_opponents_bench() {
    let (mut game, mut state) = board_with(
        CardId::A1a064PokemonFlute,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.discard_piles[1] = vec![
        get_card_by_enum(CardId::A1054Wartortle), // Stage 1: not a Basic
        get_card_by_enum(CardId::A1053Squirtle),
    ];
    game.set_state(state);

    play(&mut game, CardId::A1a064PokemonFlute);
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert!(
        state
            .enumerate_bench_pokemon(1)
            .any(|(_, p)| p.get_name() == "Squirtle"),
        "the Basic comes back to their Bench: {:?}",
        state.in_play_pokemon[1],
    );
    assert_eq!(
        state.discard_piles[1].len(),
        1,
        "and leaves the discard pile"
    );
}

/// Budding Expeditioner: "Put your Mew ex in the Active Spot into your hand."
#[test]
fn test_budding_expeditioner_only_picks_up_mew_ex() {
    for (active, expect_playable) in [
        (CardId::A1a032MewEx, true),
        (CardId::A1033Charmander, false),
    ] {
        let (mut game, state) = board_with(
            CardId::A1a066BuddingExpeditioner,
            vec![
                PlayedCard::from_id(active),
                PlayedCard::from_id(CardId::A1033Charmander),
            ],
            vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
        );
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let playable = actions.iter().any(|a| {
            matches!(&a.action, SimpleAction::Play { trainer_card }
                if trainer_card.name == "Budding Expeditioner")
        });
        assert_eq!(playable, expect_playable, "Active {active:?}");

        if playable {
            play(&mut game, CardId::A1a066BuddingExpeditioner);
            resolve_stacked_actions(&mut game);
            let state = game.get_state_clone();
            assert!(
                state.hands[0].iter().any(|c| c.get_name() == "Mew ex"),
                "Mew ex goes to hand: {:?}",
                state.hands[0],
            );
        }
    }
}

/// Blue: "During your opponent's next turn, all of your Pokémon take -10 damage from attacks from
/// your opponent's Pokémon."
#[test]
fn test_blue_softens_the_next_attack() {
    for play_blue in [true, false] {
        let (mut game, mut state) = board_with(
            CardId::A1a067Blue,
            vec![PlayedCard::new(
                get_card_by_enum(CardId::A1033Charmander),
                0,
                300,
                vec![],
                false,
                vec![],
            )],
            vec![
                PlayedCard::from_id(CardId::A1061Poliwrath).with_energy(vec![
                    EnergyType::Water,
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                ]),
            ],
        );
        if !play_blue {
            state.hands[0].clear();
        }
        game.set_state(state);

        if play_blue {
            play(&mut game, CardId::A1a067Blue);
        }
        game.apply_action(&Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        });
        game.apply_action(&Action {
            actor: 1,
            action: deckgym::test_support::attack_action(CardId::A1061Poliwrath, 0),
            is_stack: false,
        });

        // Mega Punch does 80, and Charmander is Weak to Water (+20).
        let expected = if play_blue { 210 } else { 200 };
        assert_eq!(
            game.get_state_clone().get_active(0).get_remaining_hp(),
            expected,
            "played Blue = {play_blue}",
        );
    }
}

/// Team Galactic Grunt: "Put 1 random Glameow, Stunky, or Croagunk from your deck into your hand."
#[test]
fn test_team_galactic_grunt_fetches_one_of_three() {
    let (mut game, mut state) = board_with(
        CardId::A2151TeamGalacticGrunt,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.decks[0].cards = vec![
        get_card_by_enum(CardId::A1001Bulbasaur),
        get_card_by_enum(CardId::A2107Croagunk),
    ];
    game.set_state(state);

    play(&mut game, CardId::A2151TeamGalacticGrunt);

    let state = game.get_state_clone();
    assert!(
        state.hands[0].iter().any(|c| c.get_name() == "Croagunk"),
        "the named Pokemon comes to hand: {:?}",
        state.hands[0],
    );
    assert_eq!(state.decks[0].cards.len(), 1, "and leaves the deck");
}

/// Squirt Bottle: "Discard a [R] Energy from your opponent's Active Pokémon."
#[test]
fn test_squirt_bottle_takes_a_fire_energy() {
    let (mut game, state) = board_with(
        CardId::A4152SquirtBottle,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)
            .with_energy(vec![EnergyType::Fire, EnergyType::Grass])],
    );
    game.set_state(state);

    play(&mut game, CardId::A4152SquirtBottle);

    assert_eq!(
        game.get_state_clone().get_active(1).attached_energy,
        vec![EnergyType::Grass],
        "only the [R] Energy goes",
    );
}

/// Iono: "Each player shuffles the cards in their hand into their deck, then draws that many
/// cards."
#[test]
fn test_iono_redraws_both_hands() {
    let (mut game, mut state) = board_with(
        CardId::A2b069Iono,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.hands[0].push(get_card_by_enum(CardId::A1001Bulbasaur));
    state.hands[1] = vec![get_card_by_enum(CardId::A1001Bulbasaur); 4];
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1053Squirtle); 5];
    state.decks[1].cards = vec![get_card_by_enum(CardId::A1053Squirtle); 5];
    game.set_state(state);

    play(&mut game, CardId::A2b069Iono);

    let state = game.get_state_clone();
    // Iono itself is played, so 1 card goes back in and 1 comes out.
    assert_eq!(state.hands[0].len(), 1, "you redraw what you had left");
    assert_eq!(state.decks[0].cards.len(), 5, "the deck keeps its size");
    assert_eq!(state.hands[1].len(), 4, "and so does the opponent");
    assert_eq!(state.decks[1].cards.len(), 5, "on both sides");
}

/// Hitting Hammer: "Flip 2 coins. If both of them are heads, discard a random Energy from your
/// opponent's Active Pokémon."
#[test]
fn test_hitting_hammer_needs_two_heads() {
    let mut discarded = 0;
    let seed_count = 40u64;
    for seed in 0..seed_count {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A1033Charmander)],
            vec![PlayedCard::from_id(CardId::A1001Bulbasaur)
                .with_energy(vec![EnergyType::Grass, EnergyType::Grass])],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::B1215HittingHammer)];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        play(&mut game, CardId::B1215HittingHammer);
        if game.get_state_clone().get_active(1).attached_energy.len() < 2 {
            discarded += 1;
        }
    }
    assert!(
        discarded > 0 && discarded < seed_count / 2,
        "both coins must come up heads (discarded {discarded}/{seed_count})",
    );
}

/// Fishing Net: "Put a random Basic [W] Pokémon from your discard pile into your hand."
#[test]
fn test_fishing_net_pulls_a_basic_water_pokemon() {
    let (mut game, mut state) = board_with(
        CardId::A3143FishingNet,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.discard_piles[0] = vec![
        get_card_by_enum(CardId::A1054Wartortle), // Stage 1 [W]: not Basic
        get_card_by_enum(CardId::A1001Bulbasaur), // Basic [G]
        get_card_by_enum(CardId::A1053Squirtle),  // Basic [W]
    ];
    game.set_state(state);

    play(&mut game, CardId::A3143FishingNet);

    let state = game.get_state_clone();
    assert!(
        state.hands[0].iter().any(|c| c.get_name() == "Squirtle"),
        "the Basic [W] comes to hand: {:?}",
        state.hands[0],
    );
    assert!(
        !state.discard_piles[0]
            .iter()
            .any(|c| c.get_name() == "Squirtle"),
        "and leaves the discard pile: {:?}",
        state.discard_piles[0],
    );
}

/// Acerola: "Choose 1 of your Palossand or Mimikyu that has damage on it, and move 40 of its
/// damage to your opponent's Active Pokémon."
#[test]
fn test_acerola_moves_damage_to_the_defender() {
    let (mut game, state) = board_with(
        CardId::A3148Acerola,
        vec![PlayedCard::from_id(CardId::A3083Mimikyu).with_damage(50)],
        vec![PlayedCard::new(
            get_card_by_enum(CardId::A1001Bulbasaur),
            0,
            300,
            vec![],
            false,
            vec![],
        )],
    );
    game.set_state(state);

    play(&mut game, CardId::A3148Acerola);
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        60,
        "Mimikyu keeps 10 of its 50 damage (70 HP printed)",
    );
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        260,
        "and the 40 lands on the opponent",
    );
}

/// Sophocles: "During this turn, attacks used by your Alolan Golem, Vikavolt, or Togedemaru do
/// +30 damage to your opponent's Active Pokémon."
#[test]
fn test_sophocles_boosts_the_named_three() {
    for (attacker, expected_hp) in [
        (CardId::A3065Vikavolt, 200u32), // named: 70 + 30
        // Not named: Araquanid's own 60, plus its own +60 against a Basic defender.
        (CardId::A3053Araquanid, 180),
    ] {
        let (mut game, mut state) = board_with(
            CardId::A3153Sophocles,
            vec![PlayedCard::from_id(attacker).with_energy(vec![
                EnergyType::Lightning,
                EnergyType::Lightning,
                EnergyType::Water,
                EnergyType::Colorless,
            ])],
            vec![PlayedCard::new(
                get_card_by_enum(CardId::A1001Bulbasaur),
                0,
                300,
                vec![],
                false,
                vec![],
            )],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A3153Sophocles)];
        game.set_state(state);

        play(&mut game, CardId::A3153Sophocles);
        game.apply_action(&Action {
            actor: 0,
            action: deckgym::test_support::attack_action(attacker, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "attacker {attacker:?}",
        );
    }
}

/// Whitney: "Heal 60 damage from 1 of your Miltank, and it recovers from being Asleep, Paralyzed,
/// and Confused."
#[test]
fn test_whitney_heals_and_wakes_a_miltank() {
    let (mut game, mut state) = board_with(
        CardId::A4a069Whitney,
        vec![PlayedCard::from_id(CardId::A4a062Miltank).with_damage(80)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.apply_status_condition(0, 0, deckgym::models::StatusCondition::Asleep);
    game.set_state(state);

    play(&mut game, CardId::A4a069Whitney);
    resolve_stacked_actions(&mut game);

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        90,
        "60 of the 80 damage is healed (110 HP printed)",
    );
    assert!(!state.get_active(0).is_asleep(), "and it wakes up");
}

/// Traveling Merchant: "Look at the top 4 cards of your deck. Put all Pokémon Tool cards you find
/// there into your hand."
#[test]
fn test_traveling_merchant_takes_every_tool_in_the_top_four() {
    let (mut game, mut state) = board_with(
        CardId::A4a070TravelingMerchant,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.decks[0].cards = vec![
        get_card_by_enum(CardId::A2148RockyHelmet),
        get_card_by_enum(CardId::A1001Bulbasaur),
        get_card_by_enum(CardId::A3146PoisonBarb),
        get_card_by_enum(CardId::A1001Bulbasaur),
        // Out of reach.
        get_card_by_enum(CardId::A2147GiantCape),
    ];
    game.set_state(state);

    play(&mut game, CardId::A4a070TravelingMerchant);

    let state = game.get_state_clone();
    let tools = state.hands[0]
        .iter()
        .filter(|c| c.get_name() == "Rocky Helmet" || c.get_name() == "Poison Barb")
        .count();
    assert_eq!(tools, 2, "both Tools come to hand: {:?}", state.hands[0]);
    assert_eq!(state.decks[0].cards.len(), 3, "the rest go back");
}

/// Lt. Surge: "Move all [L] Energy from your Benched Pokémon to your Raichu, Electrode, or
/// Electabuzz in the Active Spot."
#[test]
fn test_lt_surge_gathers_lightning_on_the_named_active() {
    for (active, expect_gather) in [
        (CardId::A1100Electrode, true),
        (CardId::A1033Charmander, false),
    ] {
        let (mut game, mut state) = board_with(
            CardId::A1226LtSurge,
            vec![
                PlayedCard::from_id(active),
                PlayedCard::from_id(CardId::A1094Pikachu)
                    .with_energy(vec![EnergyType::Lightning, EnergyType::Colorless]),
            ],
            vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A1226LtSurge)];
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let playable = actions.iter().any(|a| {
            matches!(&a.action, SimpleAction::Play { trainer_card } if trainer_card.name == "Lt. Surge")
        });
        assert_eq!(playable, expect_gather, "Active {active:?}");
        if !playable {
            continue;
        }

        play(&mut game, CardId::A1226LtSurge);
        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(0).attached_energy,
            vec![EnergyType::Lightning],
            "only the [L] Energy moves up",
        );
        assert_eq!(
            state.in_play_pokemon[0][1]
                .as_ref()
                .expect("Pikachu is there")
                .attached_energy,
            vec![EnergyType::Colorless],
            "and the rest stays on the Bench",
        );
    }
}

/// Juggler: "You can use this card only if your Pokémon in play have 3 or more different types of
/// Energy attached. Move all Energy from each of your Benched Pokémon to your Active Pokémon."
#[test]
fn test_juggler_needs_three_types_then_gathers_everything() {
    for (bench_energy, expect_playable) in [
        (vec![EnergyType::Water, EnergyType::Grass], true),
        (vec![EnergyType::Water], false),
    ] {
        let (mut game, mut state) = board_with(
            CardId::B2151Juggler,
            vec![
                PlayedCard::from_id(CardId::A1033Charmander).with_energy(vec![EnergyType::Fire]),
                PlayedCard::from_id(CardId::A1001Bulbasaur).with_energy(bench_energy.clone()),
            ],
            vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::B2151Juggler)];
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let playable = actions.iter().any(|a| {
            matches!(&a.action, SimpleAction::Play { trainer_card } if trainer_card.name == "Juggler")
        });
        assert_eq!(playable, expect_playable, "Bench holding {bench_energy:?}");
        if !playable {
            continue;
        }

        play(&mut game, CardId::B2151Juggler);
        let state = game.get_state_clone();
        assert_eq!(
            state.get_active(0).attached_energy.len(),
            3,
            "everything ends up on the Active Pokemon",
        );
        assert!(
            state.in_play_pokemon[0][1]
                .as_ref()
                .expect("the Benched Pokemon is there")
                .attached_energy
                .is_empty(),
            "and the Bench is empty",
        );
    }
}

/// Beast Wall: "You can use this card only if your opponent hasn't gotten any points. During your
/// opponent's next turn, all of your Ultra Beasts take -20 damage from attacks."
#[test]
fn test_beast_wall_needs_a_pointless_opponent() {
    for (their_points, expect_playable) in [(0u8, true), (1, false)] {
        let (mut game, mut state) = board_with(
            CardId::A3a063BeastWall,
            vec![PlayedCard::from_id(CardId::A3a043GuzzlordEx)],
            vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
        );
        state.points[1] = their_points;
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let playable = actions.iter().any(|a| {
            matches!(&a.action, SimpleAction::Play { trainer_card } if trainer_card.name == "Beast Wall")
        });
        assert_eq!(
            playable, expect_playable,
            "opponent on {their_points} points"
        );
    }
}

/// Fisher: "Flip 3 coins. For each heads, a [W] Pokémon is chosen at random from your discard pile
/// and put into your hand."
#[test]
fn test_fisher_reels_in_one_per_heads() {
    let mut totals = std::collections::HashSet::new();
    for seed in 0..24u64 {
        let mut game = get_initialized_game(seed);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A1033Charmander)],
            vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A4159Fisher)];
        state.discard_piles[0] = vec![get_card_by_enum(CardId::A1053Squirtle); 4];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        play(&mut game, CardId::A4159Fisher);
        totals.insert(game.get_state_clone().hands[0].len());
    }
    assert!(
        totals.iter().copied().max().unwrap() <= 3,
        "at most 3 come back: {totals:?}",
    );
    assert!(totals.len() > 1, "the count follows the coins: {totals:?}");
}

/// Prank Spinner: "A card from among both player's hands is chosen at random, revealed to the
/// other player, and shuffled into its owner's deck."
#[test]
fn test_prank_spinner_sends_one_card_back_from_each_hand() {
    let (mut game, mut state) = board_with(
        CardId::B1213PrankSpinner,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.hands[0].push(get_card_by_enum(CardId::A1053Squirtle));
    state.hands[1] = vec![get_card_by_enum(CardId::A1053Squirtle); 3];
    state.decks[0].cards = vec![];
    state.decks[1].cards = vec![];
    game.set_state(state);

    play(&mut game, CardId::B1213PrankSpinner);

    let state = game.get_state_clone();
    assert_eq!(state.hands[0].len(), 0, "your remaining card goes back");
    assert_eq!(state.decks[0].cards.len(), 1, "into your deck");
    assert_eq!(state.hands[1].len(), 2, "and one of theirs does too");
    assert_eq!(state.decks[1].cards.len(), 1, "into theirs");
}

/// Hala: "During your opponent's next turn, if your Hariyama or Crabominable would be Knocked Out
/// by damage from an attack, it is not Knocked Out and its remaining HP becomes 10."
#[test]
fn test_hala_leaves_hariyama_at_10() {
    for play_hala in [true, false] {
        let (mut game, mut state) = board_with(
            CardId::B1222Hala,
            vec![PlayedCard::from_id(CardId::A3091Hariyama)],
            vec![
                PlayedCard::from_id(CardId::A1061Poliwrath).with_energy(vec![
                    EnergyType::Water,
                    EnergyType::Colorless,
                    EnergyType::Colorless,
                ]),
            ],
        );
        if !play_hala {
            state.hands[0].clear();
        }
        // Hariyama has 120 HP printed; the 80 that follows would knock it out from 60 down.
        state.in_play_pokemon[0][0] =
            Some(PlayedCard::from_id(CardId::A3091Hariyama).with_damage(60));
        game.set_state(state);

        if play_hala {
            play(&mut game, CardId::B1222Hala);
        }
        game.apply_action(&Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        });
        game.apply_action(&Action {
            actor: 1,
            action: deckgym::test_support::attack_action(CardId::A1061Poliwrath, 0),
            is_stack: false,
        });

        let state = game.get_state_clone();
        if play_hala {
            assert_eq!(
                state.get_active(0).get_remaining_hp(),
                10,
                "Hala leaves it at 10",
            );
        } else {
            assert!(
                state.in_play_pokemon[0][0].is_none(),
                "without Hala it is knocked out",
            );
        }
    }
}

/// Penny: "Look at a random Supporter card that's not Penny from your opponent's deck and shuffle
/// it back into their deck. Use the effect of that card as the effect of this card."
#[test]
fn test_penny_borrows_a_supporter_from_their_deck() {
    let (mut game, mut state) = board_with(
        CardId::A3b069Penny,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.hands[0].clear();
    state.hands[0].push(get_card_by_enum(CardId::A3b069Penny));
    // Only Professor's Research is eligible; Penny itself is excluded by the card's own text.
    state.decks[1].cards = vec![
        get_card_by_enum(CardId::A3b069Penny),
        get_card_by_enum(CardId::PA007ProfessorsResearch),
        get_card_by_enum(CardId::A1001Bulbasaur),
    ];
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1053Squirtle); 4];
    game.set_state(state);

    play(&mut game, CardId::A3b069Penny);

    let state = game.get_state_clone();
    assert_eq!(
        state.hands[0].len(),
        2,
        "Professor's Research draws 2 for Penny's user: {:?}",
        state.hands[0],
    );
    assert_eq!(
        state.decks[1].cards.len(),
        3,
        "the opponent's deck keeps every card",
    );
}

/// Penny borrows the effect without the card ever being played, so the borrowed Supporter's own
/// "can I be played?" check is skipped. Cyrus needs a damaged Benched Pokémon on the other side;
/// without one it used to push an empty list of choices and the game had no legal move.
#[test]
fn test_penny_skips_a_supporter_that_cannot_be_played() {
    let (mut game, mut state) = board_with(
        CardId::A3b069Penny,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        // The opponent's Bench is empty, so Cyrus has nothing to switch in.
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.hands[0].clear();
    state.hands[0].push(get_card_by_enum(CardId::A3b069Penny));
    state.decks[1].cards = vec![get_card_by_enum(CardId::A2150Cyrus)];
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1053Squirtle); 4];
    game.set_state(state);

    play(&mut game, CardId::A3b069Penny);

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

/// Iono is implemented, but its B2a prints were not wired up.
#[test]
fn test_iono_b2a_print_works_too() {
    let (mut game, mut state) = board_with(
        CardId::B2a089Iono,
        vec![PlayedCard::from_id(CardId::A1033Charmander)],
        vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
    );
    state.hands[1] = vec![get_card_by_enum(CardId::A1001Bulbasaur); 4];
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1053Squirtle); 5];
    state.decks[1].cards = vec![get_card_by_enum(CardId::A1053Squirtle); 5];
    game.set_state(state);

    play(&mut game, CardId::B2a089Iono);

    let state = game.get_state_clone();
    assert_eq!(state.hands[1].len(), 4, "the opponent redraws 4");
    assert_eq!(
        state.decks[1].cards.len(),
        5,
        "and their deck keeps its size"
    );
}
