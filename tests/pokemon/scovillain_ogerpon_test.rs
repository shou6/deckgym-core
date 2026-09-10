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

fn game_with(attacker: PlayedCard, defender: PlayedCard) -> deckgym::Game<'static> {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();
    state.set_board(vec![attacker], vec![defender]);
    state.current_player = 0;
    game.set_state(state);
    game
}

fn attack(game: &mut deckgym::Game<'_>, attacker: CardId) -> u32 {
    game.apply_action(&Action {
        actor: 0,
        action: attack_action(attacker, 0),
        is_stack: false,
    });
    game.get_state_clone().get_active(1).get_remaining_hp()
}

/// Scovillain's "Red-Hot Headbutt": 60 damage, "+40 if your opponent's Active
/// Pokémon is a [G] or [M] Pokémon." Two types satisfy the same clause.
#[test]
fn test_scovillain_red_hot_headbutt_hits_grass_and_metal_harder() {
    let energy = vec![EnergyType::Fire, EnergyType::Grass, EnergyType::Colorless];

    // Bulbasaur is [G]: 60 + 40 = 100.
    let mut game = game_with(
        played_card_with_base_hp(CardId::B2a013Scovillain, 300, energy.clone()),
        played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![]),
    );
    assert_eq!(
        attack(&mut game, CardId::B2a013Scovillain),
        200,
        "a [G] defender should take 100"
    );

    // Charmander is [R]: just 60.
    let mut game = game_with(
        played_card_with_base_hp(CardId::B2a013Scovillain, 300, energy),
        played_card_with_base_hp(CardId::A1033Charmander, 300, vec![]),
    );
    assert_eq!(
        attack(&mut game, CardId::B2a013Scovillain),
        240,
        "a defender of another type should take only 60"
    );
}

/// Teal Mask Ogerpon's "Ogre's Whip": "This attack does damage to your opponent's
/// Active Pokémon equal to this Pokémon's remaining HP."
#[test]
fn test_ogerpon_ogres_whip_damage_equals_own_remaining_hp() {
    let mut game = game_with(
        PlayedCard::from_id(CardId::B4019TealMaskOgerpon)
            .with_energy(vec![EnergyType::Grass, EnergyType::Colorless])
            .with_remaining_hp(70),
        played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![]),
    );
    assert_eq!(
        attack(&mut game, CardId::B4019TealMaskOgerpon),
        230,
        "Ogerpon at 70 HP should do 70 damage"
    );

    let mut game = game_with(
        PlayedCard::from_id(CardId::B4019TealMaskOgerpon)
            .with_energy(vec![EnergyType::Grass, EnergyType::Colorless])
            .with_remaining_hp(20),
        played_card_with_base_hp(CardId::A1001Bulbasaur, 300, vec![]),
    );
    assert_eq!(
        attack(&mut game, CardId::B4019TealMaskOgerpon),
        280,
        "Ogerpon at 20 HP should do 20 damage"
    );
}
