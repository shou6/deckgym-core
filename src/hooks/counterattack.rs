use crate::{
    actions::abilities::AbilityMechanic,
    card_ids::CardId,
    effects::CardEffect,
    models::{EnergyType, PlayedCard},
    tools::has_tool,
    State,
};

/// Some cards counterattack either because of RockyHelmet or because of their own ability.
pub(crate) fn get_counterattack_damage(state: &State, card: &PlayedCard) -> u32 {
    let mut total_damage = 0;
    if has_tool(card, CardId::A2148RockyHelmet) {
        total_damage += 20;
    }

    // Temporary counterattack effects (e.g. Alolan Sandslash's Spike Armor).
    total_damage += card
        .get_active_effects()
        .iter()
        .filter_map(|effect| match effect {
            CardEffect::Counterattack { amount } => Some(*amount),
            _ => None,
        })
        .sum::<u32>();

    // Some cards have it as an ability
    if state.abilities_are_off(card) {
        return total_damage;
    }
    let card_id = CardId::from_card_id(&card.card.get_id());
    match card_id {
        Some(CardId::A1061Poliwrath)
        | Some(CardId::A1a056Druddigon)
        | Some(CardId::A2b028Pawmot)
        | Some(CardId::A3a052Ferrothorn)
        | Some(CardId::A4a065Zangoose)
        | Some(CardId::B1297Poliwrath)
        | Some(CardId::PA054Pawmot) => {
            total_damage += 20;
        }
        _ => {}
    }

    total_damage
}

/// Destiny Burst / Innards Out: a Pokémon that faints in the Active Spot from an opponent's
/// attack hits the Attacking Pokémon back. Unlike `get_counterattack_damage` this only applies
/// when the damage was lethal, so the caller must check the remaining HP first.
pub(crate) fn get_knockout_counterattack_damage(state: &State, card: &PlayedCard) -> u32 {
    match state.ability_mechanic(card) {
        Some(AbilityMechanic::CounterattackDamageOnKnockout { amount }) => *amount,
        _ => 0,
    }
}

/// Spiritomb's Final Scream: the counterpart of `get_knockout_counterattack_damage` that hits
/// every one of the attacker's Pokemon instead of just the Attacking one.
pub(crate) fn get_knockout_splash_damage(state: &State, card: &PlayedCard) -> u32 {
    match state.ability_mechanic(card) {
        Some(AbilityMechanic::DamageAllOpponentPokemonOnKnockout { amount }) => *amount,
        _ => 0,
    }
}

/// Dark Pendant: the [D] holder was hit in the Active Spot, so the attacker shuffles a random
/// card from hand back into their deck.
pub(crate) fn should_bounce_attackers_hand_card(card: &PlayedCard) -> bool {
    has_tool(card, CardId::A4154DarkPendant) && card.is_type(EnergyType::Darkness)
}

/// Check if the defending Pokemon should poison the attacker when damaged.
/// Returns true if the attacker should be poisoned.
pub(crate) fn should_poison_attacker(state: &State, card: &PlayedCard) -> bool {
    if has_tool(card, CardId::A3146PoisonBarb) {
        return true;
    }

    // Some cards have it as an ability (Dragalge ex's Poison Point)
    if state.abilities_are_off(card) {
        return false;
    }
    let card_id = CardId::from_card_id(&card.card.get_id());
    match card_id {
        Some(CardId::B1160DragalgeEx)
        | Some(CardId::B1263DragalgeEx)
        | Some(CardId::B1281DragalgeEx) => {
            return true;
        }
        _ => {}
    }

    false
}
