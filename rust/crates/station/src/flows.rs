//! Station-layer flows — the Rust port of `station.flows` (Phase-7 P7.5).
//!
//! The four cross-domain seams whose stocks belong to *different* domains, so they cannot
//! live in any `domains.*` package without one domain importing another (finding #1):
//! [`CrewRespiration`] (crew↔ECLSS gas), [`WaterRecovery`] (crew water loop), [`Lamp`]
//! (Power→biosphere energy), [`Harvest`] (biomass→food). Every `evaluate` mirrors the
//! Python arithmetic and leg-emission order character-for-character; [`CrewRespiration`]
//! **reuses** `domains::crew::carbon_split` (its Tier-1 bit-exactness depends on the
//! identical op-order — see `cabin_gas`). All four flows are transcendental-free.

use domains::biosphere::weather::PAR_UMOL_PER_J;
use domains::crew::{carbon_split, FOOD_INTAKE_VAR};
use simcore::environment::Environment;
use simcore::error::SimError;
use simcore::flow::{Flow, FlowResult, Leg};
use simcore::state::State;

/// The lamp's electrical-draw forcing var (W) — the single source both [`Lamp`] (the
/// ENERGY it withdraws) and the biosphere PAR forcing (`station::lighting`) read (#16).
pub const LAMP_POWER_VAR: &str = "lamp_power";

/// The station-owned merged respiration flow id (ASCII; str sort == Rust byte sort, #15).
pub const CREW_RESPIRATION: &str = "station.crew_respiration";
/// The station-owned water-recovery flow id.
pub const WATER_RECOVERY: &str = "station.water_recovery";
/// The station-owned grow-lamp flow id.
pub const LAMP: &str = "station.lamp";
/// The station-owned grain-harvest flow id.
pub const HARVEST: &str = "station.harvest";

/// Mean PAR-band photon energy (J per µmol photons) — the inverse of the biosphere's own
/// McCree `4.57 µmol J⁻¹`. A σ/CODATA-style module constant (not a param): the SAME
/// constant the biosphere uses to turn PAR irradiance into a photon flux, inverted here to
/// book the radiant PAR energy the lamp emits.
pub const PAR_PHOTON_ENERGY_J_PER_UMOL: f64 = 1.0 / PAR_UMOL_PER_J;

/// Read a donor stock's amount, mirroring Python `snapshot.stocks[id].amount` (a missing
/// stock is a referential-integrity error, Python's `KeyError`).
fn donor_amount(snapshot: &State, id: &str) -> Result<f64, SimError> {
    snapshot
        .stocks
        .get(id)
        .map(|s| s.amount)
        .ok_or_else(|| SimError::Reference(format!("flow reads unknown stock {id:?}")))
}

// --- CrewRespiration (P6.2) ----------------------------------------------------------

/// CARBON+OXYGEN flow `food_store + cabin_o2 → cabin_co2 + fecal_waste` (forced, 4-leg).
///
/// The atom-coupled merge of standalone crew's `OxygenConsumption` + the CO₂ leg of
/// `FoodMetabolism`. `q = env.get(crew_food_intake)·dt` is split by
/// `respired_carbon_fraction` (reusing `carbon_split`) into `respired` (→ CO₂, drawing
/// `respired` mol O₂ at PQ=1) and `feces` (egested, no O₂). Always four legs.
pub struct CrewRespiration {
    id: String,
    food_store: String,
    cabin_co2: String,
    cabin_o2: String,
    fecal_waste: String,
    respired_carbon_fraction: f64,
}

impl CrewRespiration {
    /// Construct a `CrewRespiration` with the given ids (the cabin/greenhouse seam wires
    /// `cabin_co2`/`cabin_o2` at either the ECLSS cabin pools or the biosphere gas pools).
    pub fn new(
        id: String,
        food_store: String,
        cabin_co2: String,
        cabin_o2: String,
        fecal_waste: String,
        respired_carbon_fraction: f64,
    ) -> Self {
        CrewRespiration {
            id,
            food_store,
            cabin_co2,
            cabin_o2,
            fecal_waste,
            respired_carbon_fraction,
        }
    }
}

impl Flow for CrewRespiration {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let q = env.get(FOOD_INTAKE_VAR)? * dt;
        let (respired, feces) = carbon_split(q, self.respired_carbon_fraction);
        FlowResult::new(vec![
            Leg::new(self.food_store.clone(), -q)?,
            Leg::new(self.cabin_co2.clone(), respired)?,
            Leg::new(self.cabin_o2.clone(), -respired)?,
            Leg::new(self.fecal_waste.clone(), feces)?,
        ])
    }
}

// --- WaterRecovery (P6.4) ------------------------------------------------------------

/// Station-owned water-recovery coefficients (`water_recovery.yaml`).
#[derive(Debug, Clone, Copy)]
pub struct WaterRecoveryParams {
    /// k_rec — first-order buffer draw-down rate (1/s), ≥ 0.
    pub recovery_rate: f64,
    /// η_w — recovered fraction returned to `water_store` (dimensionless), ∈ [0, 1].
    pub recovery_efficiency: f64,
}

/// Split processed water into `(potable, brine)` (kg). Op-order mirrors Python
/// `water_recovery_split`: `potable = η_w·processed`, `brine = (1 − η_w)·processed`.
fn water_recovery_split(processed_kg: f64, recovery_efficiency: f64) -> (f64, f64) {
    let potable = recovery_efficiency * processed_kg;
    let brine = (1.0 - recovery_efficiency) * processed_kg;
    (potable, brine)
}

/// WATER flow `recovered_water → water_store (+η_w) + brine (+(1−η_w))` (donor-controlled,
/// 3-leg). `processed = k_rec·recovered_water·dt`, split by η_w. Makes `water_store`
/// state-dependent (breaks the forced RK4≡Euler bit-identity) — but still only `*`/`+`/`-`
/// so Tier-1 bit-exact across ports.
pub struct WaterRecovery {
    id: String,
    recovered_water: String,
    water_store: String,
    brine: String,
    params: WaterRecoveryParams,
}

impl WaterRecovery {
    /// Construct a `WaterRecovery` with the given ids.
    pub fn new(
        id: String,
        recovered_water: String,
        water_store: String,
        brine: String,
        params: WaterRecoveryParams,
    ) -> Self {
        WaterRecovery {
            id,
            recovered_water,
            water_store,
            brine,
            params,
        }
    }
}

impl Flow for WaterRecovery {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let processed =
            self.params.recovery_rate * donor_amount(snapshot, &self.recovered_water)? * dt;
        let (potable, brine) = water_recovery_split(processed, self.params.recovery_efficiency);
        FlowResult::new(vec![
            Leg::new(self.recovered_water.clone(), -processed)?,
            Leg::new(self.water_store.clone(), potable)?,
            Leg::new(self.brine.clone(), brine)?,
        ])
    }
}

// --- Lamp (P6.5) ---------------------------------------------------------------------

/// Station-owned grow-lamp coefficient (`lamp.yaml`).
#[derive(Debug, Clone, Copy)]
pub struct LampParams {
    /// η_φ — photosynthetic photon efficacy (µmol J⁻¹) > 0.
    pub photon_efficacy: f64,
}

/// Split lamp electrical draw into `(radiant_par, waste_heat)` (J). Op-order mirrors Python
/// `lamp_energy_split`: `η_lamp = photon_efficacy·PAR_PHOTON_ENERGY_J_PER_UMOL`,
/// `radiant = η_lamp·draw`, `heat = (1 − η_lamp)·draw`.
fn lamp_energy_split(draw_joules: f64, photon_efficacy: f64) -> (f64, f64) {
    let radiant_fraction = photon_efficacy * PAR_PHOTON_ENERGY_J_PER_UMOL;
    let radiant = radiant_fraction * draw_joules;
    let heat = (1.0 - radiant_fraction) * draw_joules;
    (radiant, heat)
}

/// ENERGY flow `battery → light_used (+η_lamp) + waste_heat (+(1−η_lamp))` (forced, 3-leg).
/// `D = env.get(lamp_power)·dt`; the radiant fraction leaves as PAR light, the rest as
/// waste heat (→ `boundary.waste_heat` standalone, → `thermal.node` sealed).
pub struct Lamp {
    id: String,
    battery: String,
    light_used: String,
    waste_heat: String,
    params: LampParams,
}

impl Lamp {
    /// Construct a `Lamp` with the given ids (the sealed station re-points `waste_heat` at
    /// `thermal.node`; standalone lighting uses `boundary.waste_heat`).
    pub fn new(
        id: String,
        battery: String,
        light_used: String,
        waste_heat: String,
        params: LampParams,
    ) -> Self {
        Lamp {
            id,
            battery,
            light_used,
            waste_heat,
            params,
        }
    }
}

impl Flow for Lamp {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        _snapshot: &State,
        env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let draw = env.get(LAMP_POWER_VAR)? * dt;
        let (radiant, heat) = lamp_energy_split(draw, self.params.photon_efficacy);
        FlowResult::new(vec![
            Leg::new(self.battery.clone(), -draw)?,
            Leg::new(self.light_used.clone(), radiant)?,
            Leg::new(self.waste_heat.clone(), heat)?,
        ])
    }
}

// --- Harvest (P6.6) ------------------------------------------------------------------

/// Station-owned grain-harvest coefficient (`harvest.yaml`).
#[derive(Debug, Clone, Copy)]
pub struct HarvestParams {
    /// k_harvest — first-order grain draw rate (1/s), ≥ 0.
    pub harvest_rate: f64,
}

/// CARBON flow `storage_c → food_store` (donor-controlled, 2-leg). `harvested =
/// k_harvest·storage_c·dt`; single-currency `{CARBON:1}` transfer — no composition fold,
/// no η-split. Reads a biosphere stock, writes a crew stock (the trophic seam).
pub struct Harvest {
    id: String,
    storage_c: String,
    food_store: String,
    params: HarvestParams,
}

impl Harvest {
    /// Construct a `Harvest` with the given ids.
    pub fn new(id: String, storage_c: String, food_store: String, params: HarvestParams) -> Self {
        Harvest {
            id,
            storage_c,
            food_store,
            params,
        }
    }
}

impl Flow for Harvest {
    fn id(&self) -> &str {
        &self.id
    }
    fn evaluate(
        &self,
        snapshot: &State,
        _env: &dyn Environment,
        dt: f64,
    ) -> Result<FlowResult, SimError> {
        let harvested = self.params.harvest_rate * donor_amount(snapshot, &self.storage_c)? * dt;
        FlowResult::new(vec![
            Leg::new(self.storage_c.clone(), -harvested)?,
            Leg::new(self.food_store.clone(), harvested)?,
        ])
    }
}
