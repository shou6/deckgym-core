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

/// Rapid Strike Urshifu's "Double Type": "As long as this Pokémon is in play, it is [W] and [F]
/// type." Raichu is Weak to [F], and Urshifu is printed [W], so the Weakness only lands if the
/// second type counts.
#[test]
fn test_rapid_strike_urshifu_counts_as_fighting_for_weakness() {
    for (attacker, expected_hp) in [
        (CardId::B3051RapidStrikeUrshifu, 240u32), // 40 + 20 Weakness
        (CardId::A1053Squirtle, 280),              // a plain [W] attacker: 20, no Weakness
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(attacker)
                .with_energy(vec![EnergyType::Water, EnergyType::Colorless])],
            vec![played_card_with_base_hp(CardId::A1095Raichu, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(attacker, 0),
            is_stack: false,
        });
        // Tornado Shot also picks a Benched target; there is none here.
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "attacker {attacker:?}",
        );
    }
}

/// Lucario's "Fighting Coach" boosts "your [F] Pokémon", which now includes Rapid Strike Urshifu.
#[test]
fn test_fighting_coach_reaches_a_double_typed_urshifu() {
    for (bencher, expected_hp) in [
        (CardId::A2092Lucario, 220u32), // 40 + 20 Weakness + 20 Coach
        (CardId::A1033Charmander, 240), // 40 + 20 Weakness
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![
                PlayedCard::from_id(CardId::B3051RapidStrikeUrshifu)
                    .with_energy(vec![EnergyType::Water, EnergyType::Colorless]),
                PlayedCard::from_id(bencher),
            ],
            vec![played_card_with_base_hp(CardId::A1095Raichu, 300, vec![])],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B3051RapidStrikeUrshifu, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "Benched {bencher:?}",
        );
    }
}

/// Single Strike Urshifu is [F] and [D]: Staraptor takes -30 from [F] Pokémon, so its attack is
/// softened even though Urshifu is printed [D].
#[test]
fn test_single_strike_urshifu_counts_as_fighting_for_damage_reduction() {
    for (attacker, expected_hp) in [
        (CardId::B3113SingleStrikeUrshifu, 220u32), // 110 - 30
        (CardId::A1061Poliwrath, 220),              // 80, untouched
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        let energy = if attacker == CardId::B3113SingleStrikeUrshifu {
            vec![
                EnergyType::Darkness,
                EnergyType::Darkness,
                EnergyType::Colorless,
            ]
        } else {
            vec![
                EnergyType::Water,
                EnergyType::Colorless,
                EnergyType::Colorless,
            ]
        };
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
