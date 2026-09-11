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

/// Lum Berry: "At the end of each turn, if the Pokémon this card is attached to is affected by any
/// Special Conditions, it recovers from all of them, and discard this card."
#[test]
fn test_lum_berry_cures_at_the_end_of_the_turn() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A4091Musharna)
            .with_energy(vec![EnergyType::Psychic, EnergyType::Colorless])],
        vec![
            played_card_with_base_hp(CardId::A1033Charmander, 300, vec![])
                .with_tool(get_card_by_enum(CardId::A2149LumBerry)),
        ],
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
    assert!(
        game.get_state_clone().get_active(1).is_asleep(),
        "the defender falls asleep first",
    );

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert!(!state.get_active(1).is_asleep(), "the Berry wakes it up");
    assert!(
        state.get_active(1).attached_tools.is_empty(),
        "and is discarded",
    );
}

/// Sitrus Berry: "At the end of each turn, if the Pokémon this card is attached to has half of its
/// maximum HP or less remaining, heal 30 damage from it. If you do, discard this card."
#[test]
fn test_sitrus_berry_heals_below_half() {
    for (damage, expect_heal) in [(80u32, true), (10, false)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        // Charmander has 60 HP printed; 80 would knock it out, so use a bigger base.
        state.set_board(
            vec![PlayedCard::from_id(CardId::A1033Charmander)],
            vec![PlayedCard::new(
                get_card_by_enum(CardId::A1033Charmander),
                0,
                120,
                vec![],
                false,
                vec![],
            )
            .with_tool(get_card_by_enum(CardId::B1218SitrusBerry))
            .with_damage(damage)],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        });

        let state = game.get_state_clone();
        let expected_hp = if expect_heal {
            120 - damage + 30
        } else {
            120 - damage
        };
        assert_eq!(
            state.get_active(1).get_remaining_hp(),
            expected_hp,
            "{damage} damage on a 120 HP Pokemon",
        );
        assert_eq!(
            state.get_active(1).attached_tools.is_empty(),
            expect_heal,
            "the Berry is discarded only when it heals",
        );
    }
}

/// Rescue Scarf: "If the Pokémon this card is attached to is Knocked Out by damage from an attack
/// from your opponent's Pokémon, put it into your hand instead of the discard pile."
#[test]
fn test_rescue_scarf_returns_the_pokemon_to_hand() {
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
            played_card_with_base_hp(CardId::A1033Charmander, 60, vec![])
                .with_tool(get_card_by_enum(CardId::A4155RescueScarf)),
            PlayedCard::from_id(CardId::A1001Bulbasaur),
        ],
    );
    state.hands[1].clear();
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
        state.hands[1].iter().any(|c| c.get_name() == "Charmander"),
        "the Pokemon goes to hand: {:?}",
        state.hands[1],
    );
    assert!(
        !state.discard_piles[1]
            .iter()
            .any(|c| c.get_name() == "Charmander"),
        "and not to the discard pile: {:?}",
        state.discard_piles[1],
    );
}

/// Lucky Mittens: "Whenever your opponent's Pokémon is Knocked Out by damage from an attack used
/// by the Pokémon this card is attached to, draw a card."
#[test]
fn test_lucky_mittens_draws_on_a_knockout() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::A1061Poliwrath)
            .with_energy(vec![
                EnergyType::Water,
                EnergyType::Colorless,
                EnergyType::Colorless,
            ])
            .with_tool(get_card_by_enum(CardId::B1220LuckyMittens))],
        vec![
            played_card_with_base_hp(CardId::A1033Charmander, 60, vec![]),
            PlayedCard::from_id(CardId::A1001Bulbasaur),
        ],
    );
    state.hands[0].clear();
    state.decks[0].cards = vec![get_card_by_enum(CardId::A1001Bulbasaur); 3];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A1061Poliwrath, 0),
        is_stack: false,
    });
    while !game.get_state_clone().move_generation_stack.is_empty() {
        let (actor, choices) = game.get_state_clone().generate_possible_actions();
        game.apply_action(&Action {
            actor,
            action: choices[0].action.clone(),
            is_stack: true,
        });
    }

    assert_eq!(
        game.get_state_clone().hands[0].len(),
        1,
        "the knockout draws a card",
    );
}

/// Clear Veil: "Prevent all effects of attacks used by your opponent's Pokémon done to the Pokémon
/// this card is attached to." Regice's Crystal Body in Tool form: the damage still lands.
#[test]
fn test_clear_veil_blocks_attack_effects() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2042Aurorus).with_energy(vec![
            EnergyType::Water,
            EnergyType::Water,
            EnergyType::Colorless,
        ])],
        vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])
            .with_tool(get_card_by_enum(CardId::B4149ClearVeil))],
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
        "the 90 still lands"
    );
    assert!(
        !state.get_active(1).is_paralyzed(),
        "but the Paralysis does not",
    );
}

/// Beastite: "Attacks used by the Ultra Beast this card is attached to do +10 damage to your
/// opponent's Active Pokémon for each point you have gotten."
#[test]
fn test_beastite_scales_with_your_points() {
    for (points, expected_hp) in [(0u8, 180u32), (2, 160)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A3a043GuzzlordEx)
                .with_energy(vec![
                    EnergyType::Darkness,
                    EnergyType::Darkness,
                    EnergyType::Darkness,
                    EnergyType::Colorless,
                ])
                .with_tool(get_card_by_enum(CardId::A3a066Beastite))],
            vec![played_card_with_base_hp(CardId::A1053Squirtle, 300, vec![])],
        );
        state.points[0] = points;
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A3a043GuzzlordEx, 1),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "{points} points in hand",
        );
    }
}

/// Memory Light: "The Pokémon this card is attached to can use any attack from its previous
/// Evolutions." Celebi's Time Recall for one Pokémon.
#[test]
fn test_memory_light_unlocks_the_previous_attacks() {
    for with_tool in [true, false] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A1053Squirtle)],
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![],
            )],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A1054Wartortle)];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: SimpleAction::Evolve {
                evolution: get_card_by_enum(CardId::A1054Wartortle),
                in_play_idx: 0,
                from_deck: false,
            },
            is_stack: false,
        });

        let mut state = game.get_state_clone();
        if with_tool {
            state.in_play_pokemon[0][0]
                .as_mut()
                .unwrap()
                .attached_tools
                .push(get_card_by_enum(CardId::A4a068MemoryLight));
        }
        // Enough Energy for either attack.
        state.in_play_pokemon[0][0]
            .as_mut()
            .unwrap()
            .attached_energy = vec![EnergyType::Water, EnergyType::Water, EnergyType::Colorless];
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let offers_squirtles_attack = actions.iter().any(
            |a| matches!(&a.action, SimpleAction::Attack(attack) if attack.title == "Water Gun"),
        );
        assert_eq!(
            offers_squirtles_attack, with_tool,
            "Memory Light attached = {with_tool}",
        );
    }
}

/// Dark Pendant: "If the [D] Pokémon this card is attached to is in the Active Spot and is damaged
/// by an attack from your opponent's Pokémon, your opponent reveals a random card from their hand
/// and shuffles it into their deck."
#[test]
fn test_dark_pendant_sends_a_card_back_when_hit() {
    for (defender, expect_shuffle) in [
        (CardId::B4a041TeamRocketsMuk, true), // [D]
        (CardId::A1053Squirtle, false),       // not [D]
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
            vec![played_card_with_base_hp(defender, 300, vec![])
                .with_tool(get_card_by_enum(CardId::A4154DarkPendant))],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A1001Bulbasaur); 3];
        state.decks[0].cards = vec![];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A1061Poliwrath, 0),
            is_stack: false,
        });
        while !game.get_state_clone().move_generation_stack.is_empty() {
            let (actor, choices) = game.get_state_clone().generate_possible_actions();
            game.apply_action(&Action {
                actor,
                action: choices[0].action.clone(),
                is_stack: true,
            });
        }

        let state = game.get_state_clone();
        let expected_hand = if expect_shuffle { 2 } else { 3 };
        assert_eq!(
            state.hands[0].len(),
            expected_hand,
            "holder {defender:?}: the attacker's hand",
        );
    }
}
