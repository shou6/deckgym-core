use crate::{
    actions::{abilities::AbilityMechanic, get_ability_mechanic},
    card_ids::CardId,
    effects::{CardEffect, TurnEffect},
    models::{Card, EnergyType, PlayedCard},
    stadiums::get_peculiar_plaza_retreat_reduction,
    tools::has_tool,
    State,
};

pub(crate) fn can_retreat(state: &State) -> bool {
    let active = state.get_active(state.current_player);

    // Check if active card has CardEffect::NoRetreat
    let has_no_retreat_effect = active.get_active_effects().contains(&CardEffect::NoRetreat);

    // Check if active card is a Fossil (Fossils can never retreat)
    let is_fossil = active.is_fossil();

    !state.has_retreated && !has_no_retreat_effect && !is_fossil
}

pub(crate) fn get_retreat_cost(state: &State, card: &PlayedCard) -> Vec<EnergyType> {
    get_retreat_cost_for(state, state.current_player, card)
}

/// Retreat cost of `card`, which is in play on `owner`'s side.
///
/// Most callers ask about the player whose turn it is, but attacks such as Whimsicott ex's
/// Grass Knot ("30 more damage for each Energy in your opponent's Active Pokémon's Retreat
/// Cost") ask about the *opponent's* Active. Effects that belong to one side - the turn's
/// X Speed, benched Shaymin, the opposing Ariados - have to be resolved against `owner`,
/// not against whoever happens to be taking the turn.
pub(crate) fn get_retreat_cost_for(
    state: &State,
    owner: usize,
    card: &PlayedCard,
) -> Vec<EnergyType> {
    if let Card::Pokemon(pokemon_card) = &card.card {
        if matches!(
            get_ability_mechanic(&card.card),
            Some(AbilityMechanic::NoRetreatIfHasEnergy)
        ) && !card.attached_energy.is_empty()
        {
            return vec![];
        }
        // Heatran: free while its owner has an Arceus in play.
        if matches!(
            get_ability_mechanic(&card.card),
            Some(AbilityMechanic::NoRetreatIfArceusInPlay)
        ) && state
            .enumerate_in_play_pokemon(owner)
            .any(|(_, pokemon)| pokemon.get_name().starts_with("Arceus"))
        {
            return vec![];
        }
        // Wimpod - Wimp Out: free only while it is still its owner's first turn.
        if matches!(
            get_ability_mechanic(&card.card),
            Some(AbilityMechanic::NoRetreatOnYourFirstTurn)
        ) && state.is_users_first_turn()
        {
            return vec![];
        }
        // Jumpluff - Fluffy Flight frees its owner's Active from anywhere in play.
        if state.enumerate_in_play_pokemon(owner).any(|(_, pokemon)| {
            matches!(
                get_ability_mechanic(&pokemon.card),
                Some(AbilityMechanic::NoRetreatForYourActive)
            )
        }) {
            return vec![];
        }
        // Latios - Fantastical Floating: free while the named partner is in play.
        if let Some(AbilityMechanic::NoRetreatIfNamedPokemonInPlay { pokemon_name }) =
            get_ability_mechanic(&card.card)
        {
            if state
                .enumerate_in_play_pokemon(owner)
                .any(|(_, pokemon)| pokemon.get_name() == *pokemon_name)
            {
                return vec![];
            }
        }
        // Alolan Raichu - Surge Surfer: free while any Stadium is on the table.
        if matches!(
            get_ability_mechanic(&card.card),
            Some(AbilityMechanic::NoRetreatIfStadiumInPlay)
        ) && state.active_stadium.is_some()
        {
            return vec![];
        }
        // Tatsugiri - Retreat Directive frees its owner's Active, but only the named Pokemon.
        if state.enumerate_in_play_pokemon(owner).any(|(_, pokemon)| {
            matches!(
                get_ability_mechanic(&pokemon.card),
                Some(AbilityMechanic::NoRetreatForYourActiveNamed { pokemon_name })
                    if *pokemon_name == card.get_name()
            )
        }) {
            return vec![];
        }
        let mut normal_cost = pokemon_card.retreat_cost.clone();
        let retreat_cost_increase: u8 = card
            .get_effective_card_effects()
            .iter()
            .map(|effect| match effect {
                CardEffect::IncreasedRetreatCost { amount } => *amount,
                _ => 0,
            })
            .sum();
        for _ in 0..retreat_cost_increase {
            normal_cost.push(EnergyType::Colorless);
        }
        if has_tool(card, CardId::A4a067InflatableBoat) && card.is_type(EnergyType::Water) {
            normal_cost.pop();
        }
        if has_tool(card, CardId::B2a087BigAirBalloon) && pokemon_card.stage == 2 {
            return vec![];
        }
        if has_tool(card, CardId::B3b064SmallBalloon) && pokemon_card.stage == 0 {
            normal_cost.pop();
        }
        // Implement Retreat Cost Modifiers here.
        // Turn effects (X Speed and friends) only apply to the player taking the turn.
        let mut to_subtract = if owner == state.current_player {
            state
                .get_current_turn_effects()
                .iter()
                .filter(|x| matches!(x, TurnEffect::ReducedRetreatCost { .. }))
                .map(|x| match x {
                    TurnEffect::ReducedRetreatCost { amount } => *amount,
                    _ => 0,
                })
                .sum::<u8>()
        } else {
            0
        };

        // Shaymin's Sky Support: As long as this Pokémon is on your Bench, your Active Basic Pokémon's Retreat Cost is 1 less.
        if pokemon_card.stage == 0 {
            // Only affects Basic Pokemon
            for (_idx, benched_pokemon) in state.enumerate_bench_pokemon(owner) {
                if matches!(
                    get_ability_mechanic(&benched_pokemon.card),
                    Some(
                        AbilityMechanic::ReduceRetreatCostOfYourActiveBasicFromBench { amount: 1 }
                    )
                ) {
                    to_subtract += 1;
                }
            }
        }
        for (_idx, benched_pokemon) in state.enumerate_bench_pokemon(owner) {
            if let Some(AbilityMechanic::ReduceRetreatCostOfYourActiveTypedFromBench {
                energy_type,
                amount,
            }) = get_ability_mechanic(&benched_pokemon.card)
            {
                if card.is_type(*energy_type) {
                    to_subtract += *amount as u8;
                }
            }
        }

        // Peculiar Plaza: Psychic Pokemon retreat cost is 2 less
        for energy_type in card.types() {
            to_subtract += get_peculiar_plaza_retreat_reduction(state, energy_type);
        }

        // Beldum - Conductive Body: cheaper while another Pokemon of the same name is in play.
        if let Some(AbilityMechanic::ReduceRetreatCostIfAnotherSameNameInPlay { amount }) =
            get_ability_mechanic(&card.card)
        {
            let name = card.get_name();
            let same_name = state
                .enumerate_in_play_pokemon(owner)
                .filter(|(_, pokemon)| pokemon.get_name() == name)
                .count();
            if same_name > 1 {
                to_subtract += *amount;
            }
        }

        // Retreat Effects accumulate so we add them.
        for _ in 0..to_subtract {
            normal_cost.pop(); // Remove one colorless energy from retreat cost
        }

        // Ariados Trap Territory: Your opponent's Active Pokémon's Retreat Cost is 1 more.
        // Look at the side facing `owner`, not the side facing whoever is taking the turn.
        // Each copy is its own Ability, so two Ariados add 2.
        let facing = (owner + 1) % 2;
        for (_idx, pokemon) in state.enumerate_in_play_pokemon(facing) {
            if matches!(
                get_ability_mechanic(&pokemon.card),
                Some(AbilityMechanic::IncreaseRetreatCostForOpponentActive { amount: 1 })
            ) {
                normal_cost.push(EnergyType::Colorless);
            }
        }

        normal_cost
    } else {
        vec![]
    }
}

// Test Colorless is wildcard when counting energy
#[cfg(test)]
mod tests {
    use crate::{
        card_ids::CardId, database::get_card_by_enum, effects::TurnEffect,
        hooks::core::to_playable_card,
    };

    use super::*;

    #[test]
    fn test_retreat_costs() {
        let state = State::default();
        let card = get_card_by_enum(CardId::A1055Blastoise);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(
            retreat_cost,
            vec![
                EnergyType::Colorless,
                EnergyType::Colorless,
                EnergyType::Colorless
            ]
        );
    }

    #[test]
    fn test_retreat_costs_with_xspeed() {
        let mut state = State::default();
        state.add_turn_effect(TurnEffect::ReducedRetreatCost { amount: 1 }, 0);
        let card = get_card_by_enum(CardId::A1055Blastoise);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(
            retreat_cost,
            vec![EnergyType::Colorless, EnergyType::Colorless]
        );
    }

    #[test]
    fn test_retreat_costs_with_two_xspeed_and_two_leafs() {
        let mut state = State::default();
        state.add_turn_effect(TurnEffect::ReducedRetreatCost { amount: 1 }, 0);
        state.add_turn_effect(TurnEffect::ReducedRetreatCost { amount: 1 }, 0);
        state.add_turn_effect(TurnEffect::ReducedRetreatCost { amount: 2 }, 0);
        let card = get_card_by_enum(CardId::A1211Snorlax);
        let playable_card = to_playable_card(&card, false);
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(retreat_cost, vec![]);
    }

    #[test]
    fn test_retreat_costs_with_inflatable_boat() {
        let state = State::default();
        let card = get_card_by_enum(CardId::A1055Blastoise);
        let mut playable_card = to_playable_card(&card, false);
        playable_card.attached_tools = vec![crate::database::get_card_by_enum(
            CardId::A4a067InflatableBoat,
        )];
        let retreat_cost = get_retreat_cost(&state, &playable_card);
        assert_eq!(
            retreat_cost,
            vec![EnergyType::Colorless, EnergyType::Colorless]
        );
    }
}
