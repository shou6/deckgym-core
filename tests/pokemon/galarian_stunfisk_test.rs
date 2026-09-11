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

/// Galarian Stunfisk's "Snapping Trap": 40 damage, "During your opponent's next turn, if this
/// Pokémon is in the Active Spot when your opponent's Active Pokémon retreats, this attack does
/// 40 damage to the new Active Pokémon."
///
/// Sets up the trap (or not), hands the turn over, and lets player 1 retreat into a 300 HP
/// Pokémon. Returns that Pokémon's remaining HP.
fn retreat_into_the_trap(set_the_trap: bool) -> u32 {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2117GalarianStunfisk)
            .with_energy(vec![EnergyType::Metal, EnergyType::Colorless])],
        vec![
            PlayedCard::from_id(CardId::A1053Squirtle)
                .with_energy(vec![EnergyType::Water, EnergyType::Water]),
            played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![]),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    if set_the_trap {
        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::B2117GalarianStunfisk, 0),
            is_stack: false,
        });
    }
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 1,
        action: SimpleAction::Retreat(1),
        is_stack: false,
    });

    game.get_state_clone().get_active(1).get_remaining_hp()
}

#[test]
fn test_snapping_trap_hits_the_pokemon_that_retreats_in() {
    assert_eq!(
        retreat_into_the_trap(true),
        260,
        "the new Active Pokemon walks into 40 damage",
    );
    assert_eq!(
        retreat_into_the_trap(false),
        300,
        "and nothing happens without the attack",
    );
}

/// The card says "retreats": a forced switch (Sabrina, Victreebel, and friends) is not a retreat,
/// so the trap stays shut.
#[test]
fn test_snapping_trap_ignores_a_forced_switch() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B2117GalarianStunfisk)
            .with_energy(vec![EnergyType::Metal, EnergyType::Colorless])],
        vec![
            PlayedCard::from_id(CardId::A1053Squirtle),
            played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![]),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2117GalarianStunfisk, 0),
        is_stack: false,
    });
    // Player 0 switches the opponent's Active out, the way Sabrina does.
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Activate {
            player: 1,
            in_play_idx: 1,
        },
        is_stack: false,
    });

    assert_eq!(
        game.get_state_clone().get_active(1).get_remaining_hp(),
        300,
        "a forced switch is not a retreat",
    );
}

/// The trap needs Galarian Stunfisk to still be in the Active Spot when the retreat happens.
#[test]
fn test_snapping_trap_needs_stunfisk_to_stay_active() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B2117GalarianStunfisk)
                .with_energy(vec![EnergyType::Metal, EnergyType::Colorless]),
            PlayedCard::from_id(CardId::A1033Charmander).with_energy(vec![EnergyType::Fire]),
        ],
        vec![
            PlayedCard::from_id(CardId::A1053Squirtle)
                .with_energy(vec![EnergyType::Water, EnergyType::Water]),
            played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![]),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2117GalarianStunfisk, 0),
        is_stack: false,
    });
    // Stunfisk steps aside before the opponent's turn.
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Retreat(1),
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 1,
        action: SimpleAction::Retreat(1),
        is_stack: false,
    });

    assert_eq!(
        game.get_state_clone().get_active(1).get_remaining_hp(),
        300,
        "the trap only works from the Active Spot",
    );
}

/// Unused import guard for `Game` in this file.
#[allow(dead_code)]
fn _unused(_: &Game) {}
