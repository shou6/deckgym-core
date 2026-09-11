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

/// Revavroom's "Dual Customization": "This Pokémon may have up to 2 Pokémon Tool cards attached
/// to it." Everything else still takes one.
#[test]
fn test_revavroom_takes_a_second_tool() {
    for (holder, expect_playable) in [
        (CardId::B4115Revavroom, true), // already holding one, and it may take another
        (CardId::A1033Charmander, false), // one is the limit
    ] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(holder).with_tool(get_card_by_enum(CardId::A2148RockyHelmet))],
            vec![PlayedCard::from_id(CardId::A1001Bulbasaur)],
        );
        state.hands[0] = vec![get_card_by_enum(CardId::A2147GiantCape)];
        state.current_player = 0;
        state.turn_count = 5;
        game.set_state(state);

        let (_, actions) = game.get_state_clone().generate_possible_actions();
        let playable = actions.iter().any(|a| {
            matches!(&a.action, SimpleAction::Play { trainer_card } if trainer_card.name == "Giant Cape")
        });
        assert_eq!(playable, expect_playable, "holder {holder:?}");
        if !playable {
            continue;
        }

        game.apply_action(&Action {
            actor: 0,
            action: SimpleAction::Play {
                trainer_card: get_card_by_enum(CardId::A2147GiantCape).as_trainer(),
            },
            is_stack: false,
        });
        // Attaching is a choice; take the only Pokemon on offer.
        let (actor, choices) = game.get_state_clone().generate_possible_actions();
        let choice = choices
            .into_iter()
            .find(|a| matches!(a.action, SimpleAction::AttachTool { .. }))
            .expect("the second Tool has somewhere to go");
        game.apply_action(&Action {
            actor,
            action: choice.action,
            is_stack: true,
        });

        assert_eq!(
            game.get_state_clone().get_active(0).attached_tools.len(),
            2,
            "Revavroom ends up carrying both",
        );
    }
}

/// Both Tools work at once: Giant Cape's +20 HP and Rocky Helmet's counterattack.
#[test]
fn test_revavroom_gets_both_tool_effects() {
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
        vec![PlayedCard::from_id(CardId::B4115Revavroom)
            .with_tool(get_card_by_enum(CardId::A2147GiantCape))
            .with_tool(get_card_by_enum(CardId::A2148RockyHelmet))],
    );
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A1061Poliwrath, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        60,
        "Giant Cape's +20 HP counts: 140 - 80",
    );
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        130,
        "and Rocky Helmet still hits back for 20 (150 HP printed)",
    );
}

/// Both Tools go to the discard pile when Revavroom is Knocked Out.
#[test]
fn test_revavroom_discards_both_tools_when_knocked_out() {
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
            played_card_with_base_hp(CardId::B4115Revavroom, 60, vec![])
                .with_tool(get_card_by_enum(CardId::A2147GiantCape))
                .with_tool(get_card_by_enum(CardId::A3146PoisonBarb)),
            PlayedCard::from_id(CardId::A1001Bulbasaur),
        ],
    );
    state.discard_piles[1].clear();
    state.current_player = 0;
    state.turn_count = 5;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::A1061Poliwrath, 0),
        is_stack: false,
    });

    let discarded = game.get_state_clone().discard_piles[1]
        .iter()
        .filter(|c| c.get_name() == "Giant Cape" || c.get_name() == "Poison Barb")
        .count();
    assert_eq!(discarded, 2, "both Tools are discarded with it");
}
