use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::{attack_action, get_test_game_with_board},
};

/// Mega Sableye ex's Cursed Jewel: "During your opponent's next turn, if this Pokémon is
/// damaged by an attack, do 40 damage to the Attacking Pokémon." When that delayed 40 damage
/// lands on the attacker, if the attacker is holding its own Rocky Helmet, that hit should
/// also bounce 20 damage from the Rocky Helmet back onto Mega Sableye ex.
///
/// Mega Sableye ex sits at 150 remaining HP. Mega Blastoise ex's Triple Bombardment (130
/// damage) alone would leave it at 20 HP, but the Rocky Helmet's reflected 20 damage (triggered
/// by Cursed Jewel's own counter-hit) should bring it down to 0 and knock it out.
///
/// Rocky Helmet is attached to Mega Blastoise ex only *after* Mega Sableye ex's own attack, so
/// the only Rocky Helmet interaction in play is the one under test — not the ordinary "Rocky
/// Helmet punishes whoever attacks its holder" case, which would otherwise also hit Mega
/// Sableye ex when it attacks into Mega Blastoise ex on turn 1.
#[test]
fn test_rocky_helmet_reflects_mega_sableye_ex_cursed_jewel_counterattack() {
    let mut game = get_test_game_with_board(
        vec![PlayedCard::from_id(CardId::B3b041MegaSableyeEx)
            .with_energy(vec![EnergyType::Darkness, EnergyType::Colorless])
            .with_remaining_hp(150)],
        vec![
            PlayedCard::from_id(CardId::B1a020MegaBlastoiseEx).with_energy(vec![
                EnergyType::Water,
                EnergyType::Water,
                EnergyType::Colorless,
            ]),
        ],
    );

    // Turn 1: Mega Sableye ex uses Cursed Jewel on Mega Blastoise ex (80 damage), and arms the
    // delayed counterattack for the opponent's next turn.
    game.apply_action(&Action {
        actor: 0,
        action: attack_action(CardId::B3b041MegaSableyeEx, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        150,
        "Mega Blastoise ex should take 80 damage from Cursed Jewel (230 - 80 = 150)"
    );
    assert_eq!(
        state.get_active(0).get_remaining_hp(),
        150,
        "Mega Sableye ex shouldn't take any damage from using its own attack"
    );

    game.apply_action(&Action {
        actor: 0,
        action: SimpleAction::EndTurn,
        is_stack: false,
    });

    // Mega Blastoise ex's controller attaches Rocky Helmet before attacking, so it's in play
    // for Cursed Jewel's counterattack but never triggered Sableye's own attack on turn 1.
    let mut state = game.get_state_clone();
    state.in_play_pokemon[1][0]
        .as_mut()
        .expect("Mega Blastoise ex should be active")
        .attached_tool = Some(get_card_by_enum(CardId::A2148RockyHelmet));
    game.set_state(state);

    // Turn 2: Mega Blastoise ex attacks Mega Sableye ex with Triple Bombardment (130 damage).
    // This triggers Cursed Jewel's delayed 40 damage back onto Mega Blastoise ex, which in turn
    // should trigger Mega Blastoise ex's own Rocky Helmet, reflecting 20 damage back onto Mega
    // Sableye ex.
    game.apply_action(&Action {
        actor: 1,
        action: attack_action(CardId::B1a020MegaBlastoiseEx, 0),
        is_stack: false,
    });

    let state = game.get_state_clone();
    assert!(
        state.in_play_pokemon[0][0].is_none(),
        "Mega Sableye ex should be knocked out: 130 (Triple Bombardment) + 20 (Rocky Helmet, \
         reflected off Cursed Jewel's counterattack) = 150 damage on a 150 HP Mega Sableye ex"
    );
    assert_eq!(
        state.points[1], 3,
        "Player 1 should have won 3 points for knocking out a Mega Pokemon"
    );
}
