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

/// Ledian's "Swift": 40 damage whose "damage isn't affected by Weakness or by any
/// effects on your opponent's Active Pokémon." Onix is Weak to [G], so a normal
/// Grass attack would add 20; Swift must not.
#[test]
fn test_ledian_swift_ignores_weakness() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![PlayedCard::from_id(CardId::B2002Ledian).with_energy(vec![EnergyType::Colorless])],
        vec![played_card_with_base_hp(CardId::A1150Onix, 300, vec![])],
    );
    state.current_player = 0;
    game.set_state(state);

    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B2002Ledian, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        260,
        "Swift should do a flat 40 to Onix, with no Weakness bonus (300 - 40 = 260)"
    );
}
