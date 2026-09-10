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

/// Hariyama's "Pivot Throw": 120 damage, then "During your opponent's next turn,
/// this Pokémon takes +50 damage from attacks." Same shape as Kommo-o's Clanging
/// Scales, only the amount differs.
#[test]
fn test_hariyama_pivot_throw_increases_damage_taken_next_turn() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![PlayedCard::from_id(CardId::B4080Hariyama).with_energy(vec![
            EnergyType::Fighting,
            EnergyType::Fighting,
            EnergyType::Colorless,
        ])],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![EnergyType::Grass, EnergyType::Colorless],
        )],
    );
    state.current_player = 0;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B4080Hariyama, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        180,
        "Bulbasaur should take 120 damage from Pivot Throw (300 - 120 = 180)"
    );

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });

    // Bulbasaur attacks Hariyama with Vine Whip (40 damage).
    game.apply_action(&Action {
        actor: 1,
        action: attack_action(CardId::A1001Bulbasaur, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    // Hariyama: 120 - (40 + 50) = 30.
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        30,
        "Hariyama should take 50 extra damage from its own Pivot Throw vulnerability"
    );
}

/// The vulnerability only lasts during the opponent's next turn.
#[test]
fn test_hariyama_pivot_throw_vulnerability_expires_after_one_turn() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![played_card_with_base_hp(
            CardId::B4080Hariyama,
            300,
            vec![
                EnergyType::Fighting,
                EnergyType::Fighting,
                EnergyType::Colorless,
            ],
        )],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![EnergyType::Grass, EnergyType::Colorless],
        )],
    );
    state.current_player = 0;
    game.set_state(state);

    for action in [
        Action {
            actor: 0,
            action: attack_action(CardId::B4080Hariyama, 0),
            is_stack: false,
        },
        Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        },
        Action {
            actor: 1,
            action: SimpleAction::EndTurn,
            is_stack: false,
        },
        Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        },
    ] {
        game.apply_action(&action);
    }

    let hariyama_hp_before = game.get_state_clone().get_active(0).get_remaining_hp();

    game.apply_action(&Action {
        actor: 1,
        action: attack_action(CardId::A1001Bulbasaur, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        hariyama_hp_before - 40,
        "Vulnerability should have expired; Hariyama should only take 40 damage"
    );
}
