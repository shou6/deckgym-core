use deckgym::{
    actions::Action,
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard, StatusCondition},
    test_support::{attack_action, get_initialized_game},
};

fn played_card_with_base_hp(card_id: CardId, base_hp: u32, energy: Vec<EnergyType>) -> PlayedCard {
    PlayedCard::new(get_card_by_enum(card_id), 0, base_hp, energy, false, vec![])
}

/// Heatmor's "Roasting Heat": "If your opponent's Active Pokémon is Burned, this
/// attack does 60 more damage."
#[test]
fn test_heatmor_roasting_heat_needs_a_burned_defender() {
    // Roasting Heat is 30, so 270 without the Burn and 210 with it.
    for (burn, expected_hp) in [(false, 270u32), (true, 210)] {
        let mut game = get_initialized_game(0);
        let mut state = game.get_state_clone();
        state.set_board(
            vec![PlayedCard::from_id(CardId::A4037Heatmor)
                .with_energy(vec![EnergyType::Fire, EnergyType::Fire])],
            // Charmander is [R]: not Weak to Fire, so no weakness bonus muddies the numbers.
            vec![played_card_with_base_hp(
                CardId::A1033Charmander,
                300,
                vec![],
            )],
        );
        state.current_player = 0;
        state.turn_count = 5;
        if burn {
            state.apply_status_condition(1, 0, StatusCondition::Burned);
        }
        game.set_state(state);

        game.apply_action(&Action {
            actor: 0,
            action: attack_action(CardId::A4037Heatmor, 0),
            is_stack: false,
        });
        assert_eq!(
            game.get_state_clone().get_active(1).get_remaining_hp(),
            expected_hp,
            "burned = {burn}"
        );
    }
}

/// Ultra Necrozma ex's "Shoegaze": "Discard the top 5 cards of each player's
/// deck." Both decks, not just the opponent's.
#[test]
fn test_ultra_necrozma_shoegaze_discards_from_both_decks() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(
        vec![
            PlayedCard::from_id(CardId::PA081UltraNecrozmaEx).with_energy(vec![
                EnergyType::Dragon,
                EnergyType::Dragon,
                EnergyType::Colorless,
            ]),
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

    let before = game.get_state_clone();
    let (own_deck, opp_deck) = (before.decks[0].cards.len(), before.decks[1].cards.len());

    game.apply_action(&Action {
        actor: 0,
        // Shoegaze is the second attack; the first (Photon Claw) has no effect.
        action: attack_action(CardId::PA081UltraNecrozmaEx, 1),
        is_stack: false,
    });

    let after = game.get_state_clone();
    assert_eq!(
        after.decks[0].cards.len(),
        own_deck - 5,
        "the attacker discards 5 of its own"
    );
    assert_eq!(
        after.decks[1].cards.len(),
        opp_deck - 5,
        "and 5 of the opponent's"
    );
}
