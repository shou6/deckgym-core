use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::get_initialized_game,
};

fn played_card_with_base_hp(card_id: CardId, base_hp: u32, energy: Vec<EnergyType>) -> PlayedCard {
    PlayedCard::new(get_card_by_enum(card_id), 0, base_hp, energy, false, vec![])
}

/// Claydol's "Heal Block": "Pokémon (both yours and your opponent's) can't be healed."
///
/// Player 0 plays a Potion (heal 20) on a Charmander that has 30 damage on it. `blocker_owner`
/// says which side puts Claydol on the Bench, if any. Returns the Charmander's remaining HP.
fn potion_with_claydol(blocker_owner: Option<usize>) -> u32 {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    let mut own =
        vec![played_card_with_base_hp(CardId::A1033Charmander, 100, vec![]).with_damage(30)];
    let mut theirs = vec![PlayedCard::from_id(CardId::A1001Bulbasaur)];
    if let Some(owner) = blocker_owner {
        if owner == 0 {
            own.push(PlayedCard::from_id(CardId::A3a031Claydol));
        } else {
            theirs.push(PlayedCard::from_id(CardId::A3a031Claydol));
        }
    }
    state.set_board(own, theirs);
    state.hands[0] = vec![get_card_by_enum(CardId::PA001Potion)];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Play {
            trainer_card: get_card_by_enum(CardId::PA001Potion).as_trainer(),
        },
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

    game.get_state_clone().get_active(0).get_remaining_hp()
}

#[test]
fn test_heal_block_stops_healing_from_either_side() {
    assert_eq!(
        potion_with_claydol(None),
        90,
        "without Claydol the Potion heals 20"
    );
    assert_eq!(
        potion_with_claydol(Some(0)),
        70,
        "your own Claydol blocks your Potion too",
    );
    assert_eq!(
        potion_with_claydol(Some(1)),
        70,
        "and so does the opponent's",
    );
}

/// Moving damage counters is not healing, so Heal Block does not stop Acerola.
#[test]
fn test_heal_block_does_not_stop_moving_damage() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::A3083Mimikyu).with_damage(50),
            PlayedCard::from_id(CardId::A3a031Claydol),
        ],
        vec![played_card_with_base_hp(
            CardId::A1001Bulbasaur,
            300,
            vec![],
        )],
    );
    state.hands[0] = vec![get_card_by_enum(CardId::A3148Acerola)];
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::Play {
            trainer_card: get_card_by_enum(CardId::A3148Acerola).as_trainer(),
        },
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
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        60,
        "Mimikyu still hands 40 of its damage over (70 HP printed)",
    );
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        260,
        "and it lands on the opponent",
    );
}
