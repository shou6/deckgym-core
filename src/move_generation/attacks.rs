use crate::{
    actions::{
        abilities::AbilityMechanic, attacks::Mechanic, has_ability_mechanic, SimpleAction,
        EFFECT_MECHANIC_MAP,
    },
    effects::CardEffect,
    hooks::{contains_energy, get_attack_cost},
    models::{Attack, EnergyType, PlayedCard},
    State,
};

pub(crate) fn generate_attack_actions(state: &State) -> Vec<SimpleAction> {
    let current_player = state.current_player;
    let mut actions = Vec::new();
    if let Some(active_pokemon) = &state.in_play_pokemon[current_player][0] {
        // Fossil cards cannot attack
        if active_pokemon.is_fossil() {
            return actions;
        }

        // Check if the active Pokémon has the CannotAttack effect
        let active_effects = active_pokemon.get_active_effects();
        let cannot_attack = active_effects
            .iter()
            .any(|effect| matches!(effect, CardEffect::CannotAttack));
        if cannot_attack {
            return actions;
        }

        let restricted_attack_names: Vec<String> = active_effects
            .iter()
            .filter_map(|effect| match effect {
                CardEffect::CannotUseAttack(attack_name) => Some(attack_name.clone()),
                _ => None,
            })
            .collect();

        // The active Pokémon's own attacks, plus any granted by Celebi's Time Recall.
        let mut available_attacks: Vec<Attack> = active_pokemon.get_attacks().clone();
        available_attacks.extend(time_recall_attacks(state, current_player, active_pokemon));

        let mut offered: Vec<Attack> = Vec::new();
        for attack in available_attacks {
            // Avoid offering an identical attack twice (e.g. an attack kept unchanged across an
            // evolution). Dedup on the whole Attack, not just the title: a previous evolution can
            // share an attack's name while differing in cost/damage/effect (e.g. Swirlix's and
            // Slurpuff's "Sweets Relay"), and those are genuinely distinct, usable attacks.
            if offered.contains(&attack) {
                continue;
            }
            if restricted_attack_names.contains(&attack.title) {
                continue;
            }
            let base_cost = alternative_attack_cost(&attack, state, current_player)
                .unwrap_or_else(|| attack.energy_required.clone());
            let modified_cost = get_attack_cost(&base_cost, state, current_player);
            if contains_energy(active_pokemon, &modified_cost, state, current_player) {
                offered.push(attack.clone());
                actions.push(SimpleAction::Attack(attack));
            }
        }
    }
    actions
}

/// Celebi's Time Recall: while a Pokémon with the ability is in play, each of your evolved
/// Pokémon can use any attack from its previous Evolutions. We only need the active Pokémon's
/// previous-evolution attacks here, since only the active Pokémon can attack. The previous
/// evolutions are the under-cards recorded on the active when it evolved (`cards_behind`).
fn time_recall_attacks(state: &State, player: usize, active_pokemon: &PlayedCard) -> Vec<Attack> {
    let time_recall_active = state
        .enumerate_in_play_pokemon(player)
        .any(|(_, pokemon)| has_ability_mechanic(&pokemon.card, &AbilityMechanic::TimeRecall));
    if !time_recall_active {
        return Vec::new();
    }

    active_pokemon
        .cards_behind
        .iter()
        .flat_map(|card| card.get_attacks())
        .collect()
}

/// Boltund's Defiant Spark and Veluza's Shedding Spiral: "this attack can be used for 1 [X]
/// Energy" while a condition holds. Returns the discounted base cost, or `None` when the attack
/// has no such clause (or the condition is not met). The usual cost modifiers still apply on top.
fn alternative_attack_cost(
    attack: &Attack,
    state: &State,
    player: usize,
) -> Option<Vec<EnergyType>> {
    let effect = attack.effect.as_deref()?;
    let (energy_type, amount) = match EFFECT_MECHANIC_MAP.get(effect)? {
        Mechanic::AlternativeCostIfSelfDamaged {
            energy_type,
            amount,
        } if state.get_active(player).is_damaged() => (energy_type, amount),
        Mechanic::AlternativeCostIfDeckEmpty {
            energy_type,
            amount,
        } if state.decks[player].cards.is_empty() => (energy_type, amount),
        _ => return None,
    };
    Some(vec![*energy_type; *amount])
}
