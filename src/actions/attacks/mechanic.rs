use crate::{
    effects::{CardEffect, TurnEffect},
    models::{EnergyType, StatusCondition, TrainerType},
};

use crate::card_ids::CardId;

#[derive(Debug, Clone, PartialEq)]
pub enum BenchSide {
    YourBench,
    OpponentBench,
    BothBenches,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CopyAttackSource {
    OpponentActive,
    OpponentInPlay,
    OwnBenchNonEx,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mechanic {
    SelfHeal {
        amount: u32,
    },
    HealOneYourPokemon {
        amount: u32,
    },
    HealOneYourBenchedPokemon {
        amount: u32,
    },
    HealAllYourPokemon {
        amount: u32,
    },
    /// Diancie's Diamond Storm: heal `amount` from each of your Pokémon of `energy_type`.
    HealAllYourTypedPokemon {
        energy_type: EnergyType,
        amount: u32,
    },
    /// Mimikyu's Shadow Hit: the attack also puts `damage` on 1 of your own Pokémon (your choice).
    AlsoDamageOneOfYours {
        damage: u32,
    },
    /// Musharna's Dream Dance: both Active Pokémon come down with the same conditions.
    InflictStatusOnBothActive {
        conditions: Vec<StatusCondition>,
    },
    /// Mew's Psy Report and Unown's CHECK: the card only reveals information, which this engine
    /// has in full already. Modeled as a no-op so the card is playable.
    RevealOnly,
    /// Heal `amount` from each Benched Pokémon; if `only_basic` is true, only Basic Pokémon
    /// (Alomomola heals all, Ho-Oh heals only Basic).
    HealAllBenchedPokemon {
        amount: u32,
        only_basic: bool,
    },
    CoinFlipSelfHeal {
        amount: u32,
    },
    /// Cradily's Stick and Absorb: deal damage, heal `heal_amount` from the attacking Pokémon, then
    /// apply a `CardEffect` to an Active Pokémon (`opponent: true` → the Defending Pokémon).
    /// `SelfHeal` plus `DamageAndCardEffect` in one attack.
    SelfHealAndCardEffect {
        heal_amount: u32,
        opponent: bool,
        effect: CardEffect,
        duration: u8,
    },
    SearchToHandByEnergy {
        energy_type: EnergyType,
    },
    SearchToBenchByName {
        name: String,
    },
    /// Silcoon's & Cascoon's Cocoon Collector: "Put 3 random cards from among Silcoon and Cascoon
    /// from your deck onto your Bench." Puts up to `count` random cards named after any of `names`
    /// onto the Bench, limited by how many are in the deck and by the available Bench space.
    SearchToBenchByNames {
        names: Vec<String>,
        count: usize,
    },
    SearchToBenchBasic,
    SearchRandomPokemonToHand,
    SearchToHandByEvolvesFrom {
        name: String,
    },
    SearchToHandSupporterCard,
    /// Put a random Item card from your discard pile into your hand
    /// (Team Rocket's Slowpoke's Scavenge).
    RecoverItemFromDiscardPile,
    /// Roserade - Poison Ring: inflict status conditions on the defender and put
    /// a CardEffect on it for `effect_duration` turns.
    InflictStatusAndCardEffect {
        conditions: Vec<StatusCondition>,
        effect: CardEffect,
        effect_duration: u8,
    },
    /// Accelgor - Deck and Cover: inflict status conditions on the defender, then
    /// shuffle the attacker back into its owner's deck. Not optional.
    InflictStatusAndShuffleSelfIntoDeck {
        conditions: Vec<StatusCondition>,
    },
    InflictStatusConditions {
        conditions: Vec<StatusCondition>,
        target_opponent: bool,
    },
    InflictStatusConditionsOnBothActive {
        conditions: Vec<StatusCondition>,
    },
    ChanceStatusAttack {
        condition: StatusCondition,
    },
    /// Drampa's Dragon Breath: flip a coin; on tails this attack does nothing (no damage either).
    /// On heads, deal the attack's fixed damage and inflict `status` on the opponent's Active.
    CoinFlipNoDamageOrStatusAttack {
        status: StatusCondition,
    },
    /// Flip a coin; heads inflicts `heads_status`, tails inflicts `tails_status`, both on the
    /// opponent's Active (e.g. Lanturn ex).
    CoinFlipStatusOutcome {
        heads_status: StatusCondition,
        tails_status: StatusCondition,
    },
    /// Flip a coin; heads inflicts ALL of `conditions` on the opponent's Active (e.g.
    /// Tentacruel's Poisoned and Paralyzed).
    ChanceMultipleStatusAttack {
        conditions: Vec<StatusCondition>,
    },
    /// Flip a coin; heads inflicts `status` on the opponent's Active, tails inflicts it on the
    /// attacking Pokémon itself (e.g. Psyduck's Confusion Wave).
    CoinFlipStatusSelfOrOpponent {
        status: StatusCondition,
    },
    /// Deal damage, then let the player choose one of these Special Conditions to
    /// inflict on the opponent's Active Pokémon (e.g. Dustox's Select Powder).
    ChooseStatusToInflict {
        options: Vec<StatusCondition>,
    },
    DamageAllOpponentPokemon {
        damage: u32,
    },
    DiscardRandomGlobalEnergy {
        count: usize,
    },
    /// Porygon-Z's Buggy Beam: replace the Energy queued up in the opponent's Energy Zone with a
    /// uniformly random one of the 8 basic Energy types, regardless of their deck's Energy.
    RandomizeOpponentNextEnergy,
    RandomDamageToOpponentPokemonPerSelfEnergy {
        energy_type: EnergyType,
        damage_per_hit: u32,
    },
    DiscardEnergyFromOpponentActive,
    /// Discard one `energy_type` Energy from the opponent's Active (e.g. Dedenne, Surskit).
    DiscardOpponentActiveEnergyOfType {
        energy_type: EnergyType,
    },
    /// Discard a random Energy from BOTH Active Pokémon (e.g. Oricorio, Yveltal).
    DiscardRandomEnergyBothActive,
    CoinFlipDiscardEnergyFromOpponentActive,
    /// Pidgeot's Twister / Mega Pidgeot ex's Giant Twister: flip `num_coins` coins and discard a
    /// random Energy from the opponent's Active Pokémon for each heads. If every coin is tails
    /// the attack does nothing at all — not even its `fixed_damage`.
    CoinFlipsDiscardEnergyFromOpponentActiveOrNothing {
        num_coins: usize,
    },
    /// Maushold - Triple Gnawing: flip 'num_coins'; for each heads, discard a
    /// random Energy form the opponent's Active Pokémon. Damage always applies.
    CoinFlipsDiscardEnergyFromOpponentActive {
        num_coins: usize,
    },
    DiscardOpponentActiveToolsBeforeDamage,
    ExtraDamageIfEx {
        extra_damage: u32,
    },
    ExtraDamageIfDefenderType {
        energy_type: EnergyType,
        extra_damage: u32,
    },
    ExtraDamageIfOpponentHasSpecialCondition {
        extra_damage: u32,
    },
    ExtraDamageIfSupportPlayedThisTurn {
        extra_damage: u32,
    },
    SelfDamage {
        amount: u32,
    },
    CoinFlipExtraDamage {
        extra_damage: u32,
    },
    CoinFlipExtraDamageOrSelfDamage {
        extra_damage: u32,
        self_damage: u32,
    },
    /// Flip a coin; heads deals `damage` to the opponent's Active, tails heals `heal` from it
    /// (e.g. Delibird's Present).
    CoinFlipDamageOrHealOpponent {
        damage: u32,
        heal: u32,
    },
    CoinFlipSelfDamage {
        self_damage: u32,
    },
    /// Flip a coin; on tails discard `count` random Energy from the attacking Pokémon
    /// (e.g. Entei).
    CoinFlipSelfDiscardRandomEnergy {
        count: usize,
    },
    ExtraDamageForEachHeads {
        include_fixed_damage: bool,
        damage_per_head: u32,
        num_coins: usize,
    },
    DiscardSelfEnergyPerHeadsExtraDamage {
        num_coins: usize,
        energy_type: EnergyType,
        damage_per_discarded_energy: u32,
    },
    CoinFlipNoEffect,
    SelfDiscardEnergy {
        energies: Vec<EnergyType>,
    },
    SelfDiscardEnergyAndInflictStatus {
        energies: Vec<EnergyType>,
        conditions: Vec<StatusCondition>,
    },
    /// Kyogre's Tidal Blast: pay `energies` off this Pokémon, then spread `damage` over every
    /// one of the opponent's Pokémon.
    SelfDiscardEnergyAndDamageAllOpponent {
        energies: Vec<EnergyType>,
        damage: u32,
    },
    /// Rapid Strike Urshifu's Tornado Shot: pay `energies` off this Pokémon, and also hit 1 of
    /// the opponent's Benched Pokémon (the attacker's choice) for `bench_damage`.
    SelfDiscardEnergyAndChoiceBenchDamage {
        energies: Vec<EnergyType>,
        bench_damage: u32,
    },
    SelfDiscardEnergyAndCardEffect {
        energies: Vec<EnergyType>,
        effect: CardEffect,
        duration: u8,
    },
    /// Gouging Fire's Scorching Interruption: discard `count` random Energy from the attacking
    /// Pokémon, then leave a `CardEffect` on it (e.g. reduced damage taken next turn).
    SelfDiscardRandomEnergyAndCardEffect {
        count: usize,
        effect: CardEffect,
        duration: u8,
    },
    ExtraDamageIfExtraEnergy {
        required_extra_energy: Vec<EnergyType>,
        extra_damage: u32,
    },
    ExtraDamageIfDifferentEnergyTypesAttached {
        minimum_types: usize,
        extra_damage: u32,
    },
    /// Grafaiai's Colorful Attack: like `ExtraDamageIfDifferentEnergyTypesAttached`, but the
    /// types are counted across all of this player's Pokémon in play.
    ExtraDamageIfDifferentEnergyTypesInPlay {
        minimum_types: usize,
        extra_damage: u32,
    },
    ExtraDamageIfTypeEnergyInPlay {
        energy_type: EnergyType,
        minimum_count: usize,
        extra_damage: u32,
    },
    /// Medicham's "Psykick" / Mega Medicham ex's "Chakra Fist": extra damage if the attacking
    /// Pokémon has any Energy of `energy_type` attached. (Chakra Fist additionally shares Sawk's
    /// "isn't affected by any effects on your opponent's Active Pokémon" clause, which is detected
    /// separately from the attack's effect text in `hooks::modify_damage`.)
    ExtraDamageIfSelfHasTypeEnergy {
        energy_type: EnergyType,
        extra_damage: u32,
    },
    ExtraDamageIfStadiumInPlay {
        extra_damage: u32,
    },
    ExtraDamageIfBothHeads {
        extra_damage: u32,
    },
    DirectDamage {
        damage: u32,
        bench_only: bool,
    },
    /// Gigalith ex's Megaton Cannon: `DirectDamage` that additionally leaves a `CardEffect` on the
    /// attacking Pokémon (e.g. "During your next turn, this Pokémon can't attack.").
    DirectDamageAndSelfCardEffect {
        damage: u32,
        bench_only: bool,
        effect: CardEffect,
        duration: u8,
    },
    DamageAndTurnEffect {
        effect: TurnEffect,
        duration: u8,
    },
    SelfChargeActive {
        energies: Vec<EnergyType>,
    },
    CoinFlipSelfChargeActive {
        energies: Vec<EnergyType>,
    },
    ChargeYourTypeAnyWay {
        energy_type: EnergyType,
        count: usize,
    },
    /// Team Rocket's Moltres ex's Heat Charged: flip `num_coins` coins; for each heads, produce
    /// an Energy of `energy_type` from the Energy Zone and attach it to the attacking Pokémon
    /// itself.
    CoinFlipsAttachEnergyToSelf {
        num_coins: usize,
        energy_type: EnergyType,
    },
    // Fairly unique mechanics
    /// Manaphy's Oceanic Gift / Carbink's Glittering Gift: choose 2 of your Benched Pokémon and
    /// attach an Energy of the given type to each.
    AttachEnergyFromZoneToTwoBenched {
        energy_type: EnergyType,
    },
    PalkiaExDimensionalStorm,
    MegaKangaskhanExDoublePunchingFamily,
    MoltresExInfernoDance,
    CelebiExPowerfulBloom,
    CoinFlipPerSpecificEnergyType {
        energy_type: EnergyType,
        include_fixed_damage: bool,
        damage_per_heads: u32,
    },
    MagikarpWaterfallEvolution,
    CoinFlipToBlockAttackNextTurn,
    MoveAllEnergyTypeToBench {
        energy_type: EnergyType,
    },
    /// Swanna's Feathery Cyclone: move every Energy on this Pokémon (any type) to 1 of your
    /// Benched Pokémon.
    MoveAllEnergyToBench,
    /// Regice's Reflect Energy: move `count` random Energy from this Pokémon to 1 of your
    /// Benched Pokémon.
    MoveRandomEnergyToBench {
        count: usize,
    },
    /// Ting-Lu's Arrogant Impact: the attack does nothing while this Pokémon is down to
    /// `threshold` HP or less.
    NothingIfSelfHpAtMost {
        threshold: u32,
    },
    /// Boltund's Defiant Spark: "If this Pokémon has damage on it, this attack can be used for
    /// `amount` [X] Energy." Only the cost changes; the damage is the attack's own.
    AlternativeCostIfSelfDamaged {
        energy_type: EnergyType,
        amount: usize,
    },
    /// Veluza's Shedding Spiral: the same, but the discount needs an empty deck.
    AlternativeCostIfDeckEmpty {
        energy_type: EnergyType,
        amount: usize,
    },
    MoveFixedEnergyTypeToBench {
        energy_type: EnergyType,
        amount: u32,
    },
    ChargeBench {
        energies: Vec<EnergyType>,
        target_benched_type: Option<EnergyType>,
    },
    /// Ho-Oh ex's Phoenix Turbo: deal `fixed_damage`, then attach each of these Energies to your
    /// Benched Basic Pokémon "in any way you like" (each Energy is placed independently, so all on
    /// one Pokémon is allowed). Fossils count as Basic. If there is no Benched Basic Pokémon the
    /// Energy simply fizzles; the damage is still dealt.
    AttachEnergiesAnyWayToBenchedBasic {
        energies: Vec<EnergyType>,
    },
    VaporeonHyperWhirlpool,
    ConditionalBenchDamage {
        required_extra_energy: Vec<EnergyType>,
        bench_damage: u32,
        num_bench_targets: usize,
        opponent: bool,
    },
    ExtraDamageForEachHeadsWithStatus {
        include_fixed_damage: bool,
        damage_per_head: u32,
        num_coins: usize,
        status: StatusCondition,
    },
    /// Bellossom - Petal Dance: flip 'num_coins' coins; deal 'damage_per_head' per hedas,
    /// then apply 'status' to the ATTACKER (unlike the opponent-targeting variants).
    ExtraDamageForEachHeadsSelfStatus {
        num_coins: usize,
        damage_per_head: u32,
        status: StatusCondition,
    },
    /// Alolan Marowak - Burning Bonemerang: flip 'num_coins'. This attack does 70 damage
    /// for each heads. If at least 1 of them is heads, your opponent's Active Pokémon
    /// is now Burned.
    ExtraDamageForEachHeadsWithStatusAtLeast {
        num_coins: usize,
        damage_per_head: u32,
        status: StatusCondition,
        min_heads: usize,
    },
    /// Ambipom - Excited Tail: flip 'num_coins' coins; deal 'damage_per_head' per heads,
    /// but frlip 'boosted_num_coins' coins instead if the attacker has 'tool' attached.
    ExtraDamageForEachHeadsWithToolBoost {
        num_coins: usize,
        damage_per_head: u32,
        boosted_num_coins: usize,
        tool: CardId,
    },
    /// Croagunk / Toxicroak: flip one coin for each of your Pokémon in play, dealing
    /// `damage_per_head` for each heads.
    CoinFlipPerPokemonInPlay {
        damage_per_head: u32,
    },
    DamageAndMultipleCardEffects {
        opponent: bool,
        effects: Vec<CardEffect>,
        duration: u8,
    },
    DamageReducedBySelfDamage,
    ExtraDamagePerTrainerInOpponentDeck {
        damage_per_trainer: u32,
    },
    /// Extra damage for each card of a given Trainer kind in your discard pile (e.g. Chandelure's
    /// Past Friends counts Supporters, Rotom ex's Junk Spark counts Items).
    ExtraDamagePerTrainerTypeInDiscard {
        trainer_type: TrainerType,
        damage_per_card: u32,
    },
    ExtraDamagePerPokemonTypeInDiscard {
        energy_type: EnergyType,
        damage_per_pokemon: u32,
    },
    ExtraDamagePerPokemonInDiscard {
        damage_per_pokemon: u32,
    },
    ExtraDamagePerOwnPoint {
        damage_per_point: u32,
    },
    /// Hisuian Basculegion's Soul Counter: extra damage for each point the opponent scored
    /// during their last turn (unlike `ExtraDamagePerOpponentPoint`, which counts the whole game).
    ExtraDamagePerOpponentPointLastTurn {
        damage_per: u32,
    },
    ExtraDamagePerOpponentPoint {
        damage_per_point: u32,
    },
    ExtraDamageIfCardInDiscard {
        card_name: String,
        extra_damage: u32,
    },
    /// Buzzwole - Ground Beat / Pheromosa - Prelude: extra damage when a player's
    /// point total is exactly `points`. `opponent` picks whose points to read.
    ExtraDamageIfPointsExactly {
        opponent: bool,
        points: u8,
        extra_damage: u32,
    },
    /// Scovillain - Red-Hot Headbutt: extra damage when the defender is any of
    /// `energy_types`. The single-type form is `ExtraDamageIfDefenderType`.
    ExtraDamageIfDefenderAnyType {
        energy_types: Vec<EnergyType>,
        extra_damage: u32,
    },
    /// Teal Mask Ogerpon - Ogre's Whip: damage equal to this Pokemon's own
    /// remaining HP.
    DamageEqualToSelfRemainingHp,
    /// Celebi - Temporal Leaves: peel the top Evolution card off the defender and
    /// put it into its owner's hand. Energy, Tool and damage stay on what remains.
    DevolveDefenderToHand,
    DamageUnaffectedByWeakness,
    /// Sawk's Brick Break: fixed damage whose value "isn't affected by any effects on your
    /// opponent's Active Pokémon." The bypass itself is handled in `hooks::modify_damage` and the
    /// defender-prevention path via the attack's effect text; this variant just routes the attack
    /// as ordinary active damage (like `DamageUnaffectedByWeakness`).
    DamageUnaffectedByOpponentActiveEffects,
    DelayedSpotDamage {
        amount: u32,
    },
    /// Armaldo - Abyssal Drop: pay every Energy on the attacker, then mark one of
    /// the opponent's spots to be Knocked Out at the end of their next turn.
    /// The knockout is delayed damage large enough to finish anything.
    SelfDiscardAllEnergyAndDelayedKnockOut,
    // End Unique mechanics
    DamageAndCardEffect {
        opponent: bool,
        effect: CardEffect,
        duration: u8,
        coin_flip: bool, // false = always apply, true = apply on heads
    },
    /// Origin Forme Dialga's Time Mash and Hippowdon's Crashing Fangs: the damage always lands,
    /// and a *tails* leaves the effect behind. `DamageAndCardEffect`'s `coin_flip` applies the
    /// effect on heads instead.
    DamageAndCardEffectOnTails {
        opponent: bool,
        effect: CardEffect,
        duration: u8,
    },
    CoinFlipNoDamageOrDamageAndCardEffect {
        opponent: bool,
        effect: CardEffect,
        duration: u8,
    },
    DrawCard {
        amount: u8,
    },
    /// Draw a card for each of your Pokémon in play named `name` (e.g. Poochyena).
    DrawPerPokemonWithName {
        name: String,
    },
    /// Flip a coin; on heads set the opponent's Active remaining HP to `hp` (e.g. Xatu).
    CoinFlipSetOpponentHpTo {
        hp: u32,
    },
    SelfDiscardAllEnergy,
    /// Raging Bolt's Baneful Boom: discard all Energy from the attacking Pokémon, then Knock Out
    /// the opponent's Active Pokémon outright (no damage calculation involved).
    SelfDiscardAllEnergyAndKnockOutOpponentActive,
    SelfDiscardAllTypeEnergy {
        energy_type: EnergyType,
    },
    /// Mega Rayquaza ex's Mega Burst: discard every Energy of the listed types from the attacking
    /// Pokémon, dealing `damage_per_energy` for each Energy discarded in this way (the attack's
    /// `fixed_damage` is the per-Energy amount, so it is not added as a base).
    SelfDiscardAllTypesEnergyDamagePerDiscarded {
        energy_types: Vec<EnergyType>,
        damage_per_energy: u32,
    },
    SelfDiscardAllTypeEnergyAndDamageAnyOpponentPokemon {
        energy_type: EnergyType,
        damage: u32,
    },
    SelfDiscardRandomEnergy {
        count: usize,
    },
    AlsoBenchDamage {
        opponent: bool,
        damage: u32,
        must_have_energy: bool,
    },
    /// Walking Wake's Sweeping Billow: discard `count` random Energy from the attacking
    /// Pokémon, and this attack also does `bench_damage` to each of the chosen player's
    /// Benched Pokémon (opponent = true → opponent's bench).
    SelfDiscardRandomEnergyAndBenchDamage {
        count: usize,
        opponent: bool,
        bench_damage: u32,
    },
    AlsoChoiceBenchDamage {
        opponent: bool,
        damage: u32,
    },
    /// Toxtricity ex's Damaging Spark: deal the attack's fixed damage to the Defending Pokémon,
    /// then also deal `damage` to EVERY one of `opponent`'s Benched Pokémon that already has
    /// damage on it.
    AlsoBenchDamageIfDamaged {
        opponent: bool,
        damage: u32,
    },
    /// Team Rocket's Zapdos ex's Thunderclaw: deal the attack's fixed damage to the Defending
    /// Pokémon, then also let the attacker choose 1 of `opponent`'s Benched Pokémon that already
    /// has damage on it to deal `damage` to.
    AlsoChoiceBenchDamageIfDamaged {
        opponent: bool,
        damage: u32,
    },
    ExtraDamageIfHurt {
        extra_damage: u32,
        opponent: bool,
    },
    ExtraDamageIfUndamaged {
        extra_damage: u32,
    },
    /// Araquanid's Dangerous Claws: extra damage when the opponent's Active Pokémon is a Basic
    /// Pokémon.
    ExtraDamageIfDefenderIsBasic {
        extra_damage: u32,
    },
    /// Oranguru's Primate's Trap: like `DamageAndCardEffect`, but it leaves several effects.
    DamageAndCardEffects {
        opponent: bool,
        effects: Vec<CardEffect>,
        duration: u8,
    },
    /// Purugly's Interrupt: the attacker picks a card out of the opponent's hand and it goes back
    /// into their deck.
    ChooseOpponentHandCardToDeck,
    /// Smeargle's Splatter Coating: one random Energy on the defender becomes one of
    /// `energy_types`, drawn at random.
    RepaintRandomDefenderEnergy {
        energy_types: Vec<EnergyType>,
    },
    /// Dudunsparce's Sudden Drilling: the extra only lands if this Pokémon evolved from
    /// `pokemon_name` this turn.
    DiscardRandomDefenderEnergyIfEvolvedFrom {
        pokemon_name: String,
        count: usize,
    },
    /// Delcatty's Energy Blender: move any amount of Energy around your own side. Modeled as
    /// moving one Energy at a time (the attacker may also decline), which covers the common use
    /// without enumerating every redistribution.
    MoveOwnEnergyFreely,
    /// Bidoof's Super Fang: take half of what the defender has left, rounded down.
    HalveDefenderRemainingHp,
    /// Fan Rotom's Spin Storm: on heads the opponent's Active Pokémon (and everything on it) goes
    /// back to their hand.
    CoinFlipReturnDefenderToHand,
    /// Maushold's Family Beatdown: one coin per Pokémon in play named in `pokemon_names`, and
    /// `damage_per_head` for each heads.
    CoinPerNamedPokemonDamagePerHead {
        pokemon_names: Vec<String>,
        damage_per_head: u32,
    },
    /// Kangaskhan's Cross-Cut: the other half of `ExtraDamageIfDefenderIsBasic`.
    ExtraDamageIfDefenderIsEvolution {
        extra_damage: u32,
    },
    /// Aipom's Imitate: draw until your hand matches the opponent's.
    DrawUntilHandMatchesOpponent,
    /// Ludicolo's Rhythmic Steps and Luvdisc's Paired Tackle: extra damage when the attacker's
    /// own hand holds exactly one of `hand_sizes` cards.
    ExtraDamageIfHandSizeIs {
        hand_sizes: Vec<usize>,
        extra_damage: u32,
    },
    /// Team Rocket's Lapras's Ruthless Whirlpool: extra damage when this Pokémon has strictly
    /// more Energy attached than the opponent's Active Pokémon.
    ExtraDamageIfMoreEnergyThanDefender {
        extra_damage: u32,
    },
    /// Wishiwashi ex's School Storm: like `ExtraDamagePerPokemonWithNameOnBench`, but the ex
    /// counts alongside the Basic it evolves from ("your Benched Wishiwashi and Wishiwashi ex").
    ExtraDamagePerPokemonWithNameOrExOnBench {
        pokemon_name: String,
        damage_per: u32,
    },
    /// Regidrago's Draconic Slam: "If this Pokémon has damage on it, this attack does -100
    /// damage." The attack's `fixed_damage` is the undamaged-self base; `reduction` is subtracted
    /// (floored at 0) when the attacking Pokémon already has damage on it.
    ReducedDamageIfSelfDamaged {
        reduction: u32,
    },
    /// Vespiquen ex's Chase Order: "You may discard 1 of your Benched Basic [G] Pokémon. If you
    /// do, this attack does 70 more damage." The attacker chooses between the plain damage and
    /// discarding one eligible Benched Basic Pokémon for the boosted damage.
    OptionalDiscardBenchedBasicForExtraDamage {
        energy_type: EnergyType,
        extra_damage: u32,
    },
    /// Gyarados's Wild Swing: "You may discard any number of your Benched [W] Pokémon. This
    /// attack does `extra_damage` more damage for each Benched Pokémon you discarded in this
    /// way." Every subset of the eligible Benched Pokémon is offered as a choice.
    OptionalDiscardBenchedTypedForExtraDamage {
        energy_type: EnergyType,
        extra_damage: u32,
    },
    /// Slowking's Litter: "Discard up to `max` Pokémon Tool cards from your hand. This attack
    /// does `damage_per` damage for each card you discarded in this way." Discarding nothing is
    /// a legal choice, and then the attack does no damage at all.
    OptionalDiscardToolsFromHandForDamage {
        max: usize,
        damage_per: u32,
    },
    ExtraDamageIfStage2OnBench {
        extra_damage: u32,
    },
    ExtraDamageIfPokemonOnBench {
        pokemon_name: String,
        extra_damage: u32,
    },
    /// Drampa's Berserk: extra damage if any of the attacker's own Benched Pokémon already
    /// have damage on them.
    ExtraDamageIfAnyBenchedDamaged {
        extra_damage: u32,
    },
    /// Nidoqueen - Lovestrike: +'damage_per' for EACH benched Pokémon named 'pokemon_name'
    /// (unlike 'ExtraDamageIfPokemonBench', which is a flat bonus for a single presence).
    ExtraDamagePerPokemonWithNameOnBench {
        pokemon_name: String,
        damage_per: u32,
    },
    DamageEqualToSelfDamage,
    ExtraDamageEqualToSelfDamage,
    /// Marshadow's Revenge and friends: extra damage if any of your Pokemon were Knocked Out by
    /// an attack during the opponent's last turn. `energy_type` restricts which of your Pokemon
    /// count (e.g. Zarude's Dark Vengeance only counts `[D]` Pokemon); `None` counts any.
    ExtraDamageIfKnockedOutLastTurn {
        energy_type: Option<EnergyType>,
        extra_damage: u32,
    },
    /// Toxtricity - Vengeful Shock: the revenge bonus also leaves the defender
    /// with status conditions.
    ExtraDamageIfKnockedOutLastTurnAndInflictStatus {
        extra_damage: u32,
        conditions: Vec<StatusCondition>,
    },
    ExtraDamageIfAttackUsedDuringOwnLastTurn {
        attack_name: String,
        extra_damage: u32,
    },
    DamagePerAttackUsedThisGame {
        attack_name: String,
        damage_per_use: u32,
    },
    /// Team Rocket's Slowking ex's Hand Kinesis: deal `damage_per_card` damage for each card in
    /// the attacker's own hand.
    DamagePerOwnHandCard {
        damage_per_card: u32,
    },
    ExtraDamageIfMovedFromBench {
        extra_damage: u32,
    },
    ExtraDamageIfEvolvedThisTurn {
        extra_damage: u32,
    },
    /// Politoed's Raid: extra damage if this Pokémon evolved *from a specific Pokémon* during
    /// this turn. Unlike `ExtraDamageIfEvolvedThisTurn` this also checks the card directly
    /// underneath, so evolving via Rare Candy (which skips the named Stage 1) does not qualify.
    ExtraDamageIfEvolvedFromThisTurn {
        pokemon_name: String,
        extra_damage: u32,
    },
    BenchCountDamage {
        include_fixed_damage: bool,
        damage_per: u32,
        energy_type: Option<EnergyType>,
        bench_side: BenchSide,
    },
    EvolutionBenchCountDamage {
        include_fixed_damage: bool,
        damage_per: u32,
    },
    ExtraDamagePerEnergy {
        include_fixed_damage: bool,
        opponent: bool,
        damage_per_energy: u32,
    },
    ExtraDamagePerEnergyType {
        damage_per_type: u32,
    },
    ExtraDamagePerRetreatCost {
        damage_per_energy: u32,
    },
    DamagePerEnergyAll {
        include_fixed_damage: bool,
        opponent: bool,
        damage_per_energy: u32,
    },
    /// Choose 1 of the opponent's Pokémon; deal damage_per_energy × (energy on that Pokémon).
    DamageToAnyOpponentPerTargetEnergy {
        damage_per_energy: u32,
    },
    DiscardHandCards {
        count: usize,
    },
    ExtraDamagePerSpecificEnergy {
        energy_type: EnergyType,
        damage_per_energy: u32,
    },
    ExtraDamagePerSpecificEnergyAllYours {
        energy_type: EnergyType,
        damage_per_energy: u32,
    },
    ExtraDamageIfToolAttached {
        extra_damage: u32,
    },
    RecoilIfKo {
        self_damage: u32,
    },
    ShuffleOpponentActiveIntoDeck,
    KnockBackOpponentActive,
    /// Random spread damage attack (e.g., Draco Meteor, Spurt Fire)
    /// Always targets opponent's active + bench. Optionally includes own bench.
    RandomSpreadDamage {
        times: usize,
        damage_per_hit: u32,
        include_own_bench: bool,
    },
    FlipUntilTailsDamage {
        damage_per_heads: u32,
    },
    /// Like `FlipUntilTailsDamage`, but the attack's `fixed_damage` is dealt as a base and each
    /// heads adds `damage_per_heads` on top (e.g. "does 30 more damage for each heads").
    FlipUntilTailsBonusDamage {
        damage_per_heads: u32,
    },
    DirectDamageIfDamaged {
        damage: u32,
    },
    AttachEnergyToBenchedBasic {
        energy_type: EnergyType,
    },
    DamageAndDiscardOpponentDeck {
        discard_count: usize,
    },
    /// Coalossal's Mountain Crush: deal the attack's `fixed_damage`, then flip a coin until
    /// tails, discarding the top card of the opponent's deck for each heads.
    FlipUntilTailsDiscardOpponentDeck,
    /// Ultra Necrozma ex - Shoegaze: discard the top `discard_count` cards of
    /// *each* player's deck, the attacker's included.
    DamageAndDiscardBothDecks {
        discard_count: usize,
    },
    /// Kabutops - Leech Life: heal the same amount of damage dealt.
    HealEqualToDamageDealt,
    MegaAmpharosExLightningLancer,
    OminousClaw,
    DarknessClaw,
    BlockBasicAttack,
    SwitchSelfWithBench,
    MaySwitchSelfWithBench,
    /// Eldegoss - Float Up: "You may shuffle this Pokémon and all attached cards
    /// into your deck." Optional, so the attacker is offered a decline too.
    MayShuffleSelfIntoDeck,
    SelfHealIfStadiumInPlay {
        amount: u32,
    },
    InflictStatusIfStadiumInPlay {
        status: StatusCondition,
    },
    CopyAttack {
        source: CopyAttackSource,
        require_attacker_energy_match: bool,
    },
    SelfAsleepAndHeal {
        amount: u32,
    },
    /// Wailord ex's Wondrous Waves: after dealing damage, the attacking Pokémon recovers from
    /// all Special Conditions.
    SelfCureStatusConditions,
    FlipCoinsBenchDamagePerHead {
        num_coins: usize,
        bench_damage_per_head: u32,
    },
    ExtraDamageIfSelfHpAtMost {
        threshold: u32,
        extra_damage: u32,
    },
    ExtraDamageIfOpponentHpMoreThanSelf {
        extra_damage: u32,
    },
    ExtraDamageIfOpponentActiveHasAbility {
        extra_damage: u32,
    },
    /// Honchkrow – Evil Admonition: extra damage for each of the opponent's
    /// Pokémon in play (active and bench) that has an Ability.
    ExtraDamagePerOpponentPokemonWithAbility {
        damage_per: u32,
    },
    CoinFlipShuffleRandomOpponentHandCardIntoDeck,
    /// Tsareena - Kick Down: shuffle a random card from the opponent's hand
    /// into their deck. No coin flip, so it always happens.
    ShuffleRandomOpponentHandCardIntoDeck,
    /// Persian - Shadow Claw: flip a coin; if heads, discard a random card
    /// from the opponent's hand after dealing damage.
    CoinFlipDiscardRandomOpponentHandCard,
    /// Shiftry - Nipping Cyclone: discard a random card from the opponent's
    /// hand. No coin flip, so it always happens.
    DiscardRandomOpponentHandCard,
    /// Krookodile - Poaching Fangs: flip 'num_coins' cions; for each heads, shuffle
    /// a random card from the opponent's hand into their deck.
    CoinFlipsShuffleOpponentHandCards {
        num_coins: usize,
    },
    /// Teal Mask Ogerpon ex – Energized Leaves:
    /// If total energy on both Active Pokémon ≥ threshold, deal extra_damage more.
    ExtraDamageIfCombinedActiveEnergyAtLeast {
        threshold: usize,
        extra_damage: u32,
    },
    /// Hearthflame Mask Ogerpon – Hearthflame Dance:
    /// Flip a coin. If heads, take `count` energy of `energy_type` from your Energy Zone
    /// and attach them to 1 of your Benched Pokémon.
    CoinFlipChargeBench {
        energies: Vec<EnergyType>,
        target_benched_type: Option<EnergyType>,
    },
    /// Wellspring Mask Ogerpon – Wellspring Dance:
    /// Flip a coin. If heads, this attack also does `damage` to 1 of the chosen player's
    /// Benched Pokémon (opponent = true → opponent's bench).
    CoinFlipAlsoChoiceBenchDamage {
        opponent: bool,
        damage: u32,
    },
    /// Venoshock – extra damage if opponent's active is Poisoned.
    ExtraDamageIfDefenderPoisoned {
        extra_damage: u32,
    },
    /// Hatterene – Mental Crush: extra damage if opponent's active is Confused.
    /// Heatmor - Roasting Heat: extra damage when the defender is Burned.
    ExtraDamageIfDefenderBurned {
        extra_damage: u32,
    },
    /// Team Rocket's Magmar - Derisive Roasting: extra damage for every Special
    /// Condition on the defender, so Poison and Burn together count twice.
    ExtraDamagePerDefenderSpecialCondition {
        damage_per_condition: u32,
    },
    /// Magmortar - Thundering Volcano: the bench splash only happens when the
    /// named Pokemon is on the attacker's bench.
    AlsoBenchDamageIfPokemonOnBench {
        pokemon_name: String,
        bench_damage: u32,
    },
    /// Volcarona - Volcanic Ash: discard `count` Energy of `energy_type` from the
    /// attacker, then deal `damage` to any one of the opponent's Pokemon.
    SelfDiscardTypeEnergyAndDamageAnyOpponentPokemon {
        energy_type: EnergyType,
        count: usize,
        damage: u32,
    },
    /// Druddigon - Giga Claw: flip 2 coins; the attack does nothing if both are
    /// tails.
    NothingIfBothTails,
    /// Chinchou - Luring Glow: on heads the opponent's Active is replaced by one
    /// of their Benched Pokemon, chosen by the attacker.
    CoinFlipDragOpponentBench,
    /// Galvantula - Electric Shock: pay every Energy on the attacker, then apply
    /// status conditions to the defender.
    SelfDiscardAllEnergyAndInflictStatus {
        conditions: Vec<StatusCondition>,
    },
    /// Ampharos - Zapping Bullet: extra damage to one of the opponent's Benched
    /// Pokemon, picked at random rather than by the attacker.
    AlsoRandomBenchDamage {
        bench_damage: u32,
    },
    /// Tapu Koko - Volt Switch: swap the attacker for a Benched Pokemon of
    /// `energy_type`. `SwitchSelfWithBench` takes any Benched Pokemon.
    SwitchSelfWithTypedBench {
        energy_type: EnergyType,
    },
    /// Pachirisu - Crackling Snap: discard the top card of the attacker's deck and
    /// add damage when it turns out to be an Item.
    DiscardTopThenExtraDamageIfItem {
        extra_damage: u32,
    },
    /// Machop - Shatter / Conkeldurr - Bedrock Breaker: discard whatever Stadium
    /// is in play, whoever put it there.
    DiscardStadiumInPlay,
    /// Archeops - Wild Spin: damage every one of the opponent's Pokemon, and set
    /// this same attack up to hit `boost` harder next turn.
    DamageAllOpponentPokemonAndBoostSelfAttack {
        /// Wild Spin's printed damage is 0; the 20 lives in the effect text.
        damage: u32,
        attack_name: String,
        boost: u32,
    },
    /// Quagsire - Amnesia: pick one of the defender's attacks at random and lock
    /// it for the opponent's next turn.
    LockRandomDefenderAttack {
        duration: u8,
    },
    /// Sandy Shocks - Pull In and Pound: drag one of the opponent's Benched
    /// Pokemon out, then hit whatever came out. No Bench means no damage either.
    DragOpponentBenchThenDamage {
        damage: u32,
    },
    /// Golurk - Heavy Rocket: look at the top `reveal` cards and add `damage_per`
    /// for each Pokemon whose Retreat Cost is at least `retreat_cost_at_least`.
    /// The cards go back into the deck.
    RevealTopThenDamagePerHeavyPokemon {
        reveal: usize,
        retreat_cost_at_least: usize,
        damage_per: u32,
    },
    /// Dugtrio - Cliff Crumbler: discard the top card of the attacker's deck and
    /// add damage when it is a Pokemon of `energy_type`.
    DiscardTopThenExtraDamageIfTypedPokemon {
        energy_type: EnergyType,
        extra_damage: u32,
    },
    /// Tyrantrum - Tyrannical Fang: extra damage while the attacker has fewer
    /// Pokemon in play than the opponent.
    ExtraDamageIfFewerPokemonInPlay {
        extra_damage: u32,
    },
    /// Marowak - Punish: extra damage when the defender's name contains
    /// `name_part`.
    ExtraDamageIfDefenderNameContains {
        name_part: String,
        extra_damage: u32,
    },
    /// Wobbuffet's Reply Strongly: extra damage when this Pokémon was hit by an attack while
    /// Active during the opponent's last turn.
    ExtraDamageIfActiveDamagedLastTurn {
        extra_damage: u32,
    },
    /// Uxie's Mind Boost: attach one `energy_type` Energy from the Energy Zone to one of your
    /// Pokémon named in `pokemon_names` (your choice).
    AttachEnergyFromZoneToNamed {
        energy_type: EnergyType,
        pokemon_names: Vec<String>,
    },
    /// Mesprit's Supreme Blast: usable only while `pokemon_names` are all on your Bench; the
    /// attack then costs this Pokémon every Energy it carries.
    RequireBenchedNamesThenDiscardAllEnergy {
        pokemon_names: Vec<String>,
    },
    /// Mew's Miraculous Memory: one attack from among the Pokémon in the opponent's hand and deck
    /// is picked at random and used as this attack.
    RandomAttackFromOpponentHandAndDeck,
    /// Clefairy's Mini-Metronome: on heads, copy one of the defender's attacks.
    CoinFlipCopyDefenderAttack,
    /// Gothitelle's Stellar Cradle: while the effect lasts, the Defending Pokémon falls asleep if
    /// its owner attaches Energy from their Energy Zone to it.
    SleepIfDefenderIsCharged,
    /// Scream Tail's Shooing Shout: the same as `CoinFlipDiscardOpponentActive`, but every one of
    /// `flips` coins has to come up heads.
    AllHeadsDiscardOpponentActive {
        flips: usize,
    },
    /// Mime Jr.'s Mime-y Shuffle: shuffle your hand into your deck, then draw one card for each
    /// card in the opponent's hand.
    ShuffleHandThenDrawOpponentHandSize,
    /// Hoopa's Mischievous Ring: before the damage, every Pokémon Tool on the opponent's side
    /// goes back into their deck.
    ShuffleOpponentToolsIntoDeck,
    /// Guzzlord's Breakcore: on heads the opponent's Active Pokémon goes straight to the discard
    /// pile. It counts as a knockout, so the points are awarded the usual way.
    CoinFlipDiscardOpponentActive,
    /// Sableye's Jeweled Gift: take one Energy of a random type from among `energy_types` out of
    /// the Energy Zone and attach it to 1 of your Benched Pokémon (your choice).
    RandomTypedEnergyFromZoneToBenched {
        energy_types: Vec<EnergyType>,
    },
    /// Liepard's Snatch and Flee: the opponent shuffles a random card from hand into their deck,
    /// and this Pokémon shuffles itself back into its owner's deck.
    ShuffleOpponentHandCardAndSelfIntoDeck,
    /// Kingambit's Overlord's Blade: extra damage for each of your own Pokémon that has been
    /// Knocked Out this game.
    ExtraDamagePerOwnKnockout {
        damage_per: u32,
    },
    /// Alolan Raticate's Scrounge-and-Scarf and Alolan Meowth's Meddle: discard one random card
    /// of `trainer_type` from the opponent's hand.
    DiscardRandomOpponentHandTrainer {
        trainer_type: TrainerType,
    },
    /// Purrloin's Playful Knockdown: knock every Pokémon Tool off the opponent's Active Pokémon.
    DiscardToolsFromOpponentActive,
    /// Alolan Muk ex's Chemical Panic: one Special Condition the defender does not already have,
    /// picked at random.
    RandomStatusConditionToDefender {
        options: Vec<StatusCondition>,
    },
    /// Grumpig's Swaying Dance: like `ExtraDamageIfHandSizeIs`, but it counts the opponent's hand.
    ExtraDamageIfOpponentHandSizeIs {
        hand_sizes: Vec<usize>,
        extra_damage: u32,
    },
    /// Chimecho's Extrasensory: extra damage while both hands hold the same number of cards.
    ExtraDamageIfHandsAreEqual {
        extra_damage: u32,
    },
    /// Mr. Mime's Synchro Dance: extra damage while both Active Pokémon carry the same number of
    /// Energy.
    ExtraDamageIfEqualEnergyCount {
        extra_damage: u32,
    },
    /// Enamorus's Smitten Strike: extra damage while both Active Pokémon share an Energy type.
    ExtraDamageIfSharedEnergyType {
        extra_damage: u32,
    },
    /// Flutter Mane's Hexing Flight: the attack does nothing unless this Pokémon moved from the
    /// Bench to the Active Spot this turn.
    NothingUnlessMovedFromBench,
    /// Team Rocket's Wobbuffet's Rocket Frenzy: reveal the top `reveal` cards and add
    /// `damage_per` for each Pokémon among them whose name contains `name_part`, then shuffle
    /// them back.
    RevealTopThenDamagePerNamedPokemon {
        reveal: usize,
        name_part: String,
        damage_per: u32,
    },
    /// Swalot's Swallow Up: extra damage while the opponent's Active Pokémon has strictly less
    /// remaining HP than the attacker.
    ExtraDamageIfDefenderHasLessHp {
        extra_damage: u32,
    },
    /// Team Rocket's Muk's Poison Absorption: heal the attacker if the defender is Poisoned.
    HealSelfIfDefenderPoisoned {
        amount: u32,
    },
    /// Toxicroak's Toxic and Toxapex's Severe Poison: Poison the defender, and make that Poison
    /// bite for `amount` instead of the usual 10.
    PoisonWithDamageAmount {
        amount: u32,
    },
    /// Groudon - Gaia Blast: discard `count` random Energy from among everything
    /// attached to the attacker's own Pokemon.
    DiscardRandomOwnEnergy {
        count: usize,
    },
    /// Rotom - Assault Laser: extra damage while the *defender* holds a Tool.
    /// `ExtraDamageIfToolAttached` looks at the attacker's own Tool instead.
    ExtraDamageIfDefenderToolAttached {
        extra_damage: u32,
    },
    /// Bronzong - Psychic Resonance: extra damage while the opponent has any
    /// Pokemon of `energy_type` anywhere in play, Bench included.
    ExtraDamageIfOpponentHasTypeInPlay {
        energy_type: EnergyType,
        extra_damage: u32,
    },
    /// Forretress - Enormous Explosion: damage to the defender, to the attacker
    /// itself, and to every Benched Pokemon on both sides.
    SelfDamageAndAllBenchDamage {
        self_damage: u32,
        bench_damage: u32,
    },
    /// Team Rocket's Tinkaton - Pile-Driving Hammer: raise both the defender's
    /// attack cost and its Retreat Cost for the opponent's next turn.
    RaiseDefenderAttackAndRetreatCost {
        amount: u8,
        duration: u8,
    },
    /// Aegislash - Superb Shield: put a damage reduction against Pokemon ex on
    /// the attacker itself for the opponent's next turn.
    SelfReducedDamageFromEx {
        amount: u32,
        duration: u8,
    },
    ExtraDamageIfDefenderConfused {
        extra_damage: u32,
    },
    /// Breloom – Pre-Dawn Strike: extra damage if opponent's active is Asleep.
    ExtraDamageIfDefenderAsleep {
        extra_damage: u32,
    },
    /// Discard the top card of the attacker's own deck after dealing damage.
    DiscardTopSelfDeck {
        count: usize,
    },
    /// Tiered coin flip damage: flip `num_coins` coins and deal fixed_damage +
    /// extra_damage_by_heads[heads_count] total damage.
    TieredCoinFlipDamage {
        num_coins: usize,
        extra_damage_by_heads: Vec<u32>,
    },
    /// First attack after coming into play: conditionally apply a turn effect (e.g. Flutter Mane).
    FirstAttackBonusTurnEffect {
        effect: TurnEffect,
        duration: u8,
    },
    /// First attack after coming into play: conditionally deal extra damage and inflict status (e.g. Iron Bundle).
    FirstAttackBonusDamageAndStatus {
        extra_damage: u32,
        conditions: Vec<StatusCondition>,
    },
    /// Growlithe – Puppy Pile: deal damage_per × (number of own Pokémon in play and hand
    /// that have an attack named `attack_name`).
    DamagePerOwnPokemonWithAttackName {
        attack_name: String,
        damage_per: u32,
    },
    /// Emolga (Windup Thunder) / Dedenne ex (Dede-Circuit):
    /// deal `damage_per` damage for each Pokémon Tool attached to any of your
    /// Pokémon in play (active + bench).
    DamagePerOwnToolAttached {
        damage_per: u32,
    },
}
