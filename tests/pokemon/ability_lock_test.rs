use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::{attack_action, get_initialized_game},
    Game, State,
};

fn played_card_with_base_hp(card_id: CardId, base_hp: u32, energy: Vec<EnergyType>) -> PlayedCard {
    PlayedCard::new(get_card_by_enum(card_id), 0, base_hp, energy, false, vec![])
}

/// Whether `player` is offered the ability of the Pokémon in `in_play_idx`.
fn offers_ability(state: &State, in_play_idx: usize) -> bool {
    let (_, actions) = state.generate_possible_actions();
    actions.iter().any(
        |a| matches!(a.action, SimpleAction::UseAbility { in_play_idx: idx } if idx == in_play_idx),
    )
}

/// A turn opens with a forced draw sitting on the move generation stack; play it so that the
/// player's real options (abilities included) are the ones generated.
fn take_turn_start_draw(game: &mut Game) {
    let state = game.get_state_clone();
    let (actor, actions) = state.generate_possible_actions();
    assert_eq!(actions.len(), 1, "expected only the turn-start draw");
    let action = actions[0].action.clone();
    game.apply_action(&Action {
        actor,
        action,
        is_stack: true,
    });
}

/// Budew's "Prickly Powder": "The Defending Pokémon loses all Abilities. This effect lasts until
/// the Defending Pokémon leaves the Active Spot."
#[test]
fn test_prickly_powder_turns_the_defenders_ability_off() {
    for use_the_attack in [true, false] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::B3013Budew)],
            vec![
                // Porygon's Data Scan is an ability its owner may use on their turn.
                played_card_with_base_hp(CardId::A1209Porygon, 300, vec![]),
                PlayedCard::from_id(CardId::A1001Bulbasaur),
            ],
        );
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        if use_the_attack {
            game.apply_action(&Action {
                actor: 0,
                action: attack_action(CardId::B3013Budew, 0),
                is_stack: false,
            });
        }
        game.apply_action(&Action {
            actor: 0,
            action: SimpleAction::EndTurn,
            is_stack: false,
        });
        take_turn_start_draw(&mut game);

        assert_eq!(
            offers_ability(&game.get_state_clone(), 0),
            !use_the_attack,
            "used Prickly Powder = {use_the_attack}",
        );
    }
}

/// The lock follows the Pokémon out of the Active Spot: retreating clears it.
#[test]
fn test_prickly_powder_wears_off_when_the_defender_retreats() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![PlayedCard::from_id(CardId::B3013Budew)],
        vec![
            played_card_with_base_hp(CardId::A1209Porygon, 300, vec![EnergyType::Colorless]),
            PlayedCard::from_id(CardId::A1001Bulbasaur),
        ],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B3013Budew, 0),
        is_stack: false,
    });
    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });
    take_turn_start_draw(&mut game);
    // Porygon steps back to the Bench and Bulbasaur comes up.
    game.apply_action(&Action {
        actor: 1,
        action: SimpleAction::Retreat(1),
        is_stack: false,
    });

    assert!(
        offers_ability(&game.get_state_clone(), 1),
        "the Benched Porygon has its ability back",
    );
}

/// Alolan Muk's "Power of Alchemy": "Basic Pokémon in play (both yours and your opponent's) have
/// no Abilities." Evolutions keep theirs, and so does Alolan Muk itself (it is a Stage 1).
#[test]
fn test_power_of_alchemy_silences_basics_on_both_sides() {
    for muk_owner in [None, Some(0usize), Some(1usize)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        let mut own = vec![
            PlayedCard::from_id(CardId::A1209Porygon), // Basic, has an ability
            PlayedCard::from_id(CardId::A3b059Ambipom), // Stage 1, has an ability
        ];
        let mut theirs = vec![PlayedCard::from_id(CardId::A1001Bulbasaur)];
        match muk_owner {
            Some(0) => own.push(PlayedCard::from_id(CardId::B2097AlolanMuk)),
            Some(1) => theirs.push(PlayedCard::from_id(CardId::B2097AlolanMuk)),
            _ => {}
        }
        state.set_board(own, theirs);
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        let state = game.get_state_clone();
        assert_eq!(
            offers_ability(&state, 0),
            muk_owner.is_none(),
            "Porygon (Basic) with Alolan Muk owned by {muk_owner:?}",
        );
        assert!(
            offers_ability(&state, 1),
            "Ambipom (Stage 1) keeps its ability whatever Muk does",
        );
    }
}
