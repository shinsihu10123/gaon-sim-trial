#![forbid(unsafe_code)]

use simulation_core::SimulationEngine;
use simulation_protocol::RenderSnapshot;
use simulation_save::{create_bundle, SaveKind};
use simulation_worldgen::generate_trial_terrain;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SimulationWasm {
    engine: SimulationEngine,
}

#[wasm_bindgen]
impl SimulationWasm {
    /// Creates the browser-owned authoritative engine and its deterministic
    /// Stage 2.2 terrain.
    ///
    /// A 16-digit hexadecimal seed avoids JavaScript's lossy 53-bit integer
    /// conversion for arbitrary Rust `u64` seeds.
    ///
    /// # Errors
    ///
    /// Returns a JavaScript error when seed parsing, terrain generation,
    /// authoritative snapshot validation or engine restoration fails.
    #[wasm_bindgen(constructor)]
    pub fn new(seed_hex: &str) -> Result<SimulationWasm, JsValue> {
        let seed = parse_seed_hex(seed_hex).map_err(|message| JsValue::from_str(&message))?;
        let terrain =
            generate_trial_terrain(seed).map_err(|error| JsValue::from_str(&error.to_string()))?;

        let engine = SimulationEngine::new(seed);
        let mut snapshot = engine.export_snapshot();
        snapshot.world.terrain = Some(terrain);
        let bundle = create_bundle(&snapshot, SaveKind::Manual)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let engine = SimulationEngine::from_save_bundle(&bundle)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        Ok(Self { engine })
    }

    /// Serializes the current renderer-facing snapshot.
    ///
    /// # Errors
    ///
    /// Returns a JavaScript error when the authoritative digest or JSON
    /// serialization cannot be produced.
    #[wasm_bindgen(js_name = snapshotJson)]
    pub fn snapshot_json(&self) -> Result<String, JsValue> {
        self.encode_snapshot()
    }

    /// Advances the authoritative engine and returns the resulting snapshot.
    ///
    /// # Errors
    ///
    /// Returns a JavaScript error when the resulting authoritative digest or
    /// JSON serialization cannot be produced.
    #[wasm_bindgen(js_name = advanceDays)]
    pub fn advance_days(&mut self, days: u32) -> Result<String, JsValue> {
        self.engine.advance_days(u64::from(days));
        self.encode_snapshot()
    }
}

impl SimulationWasm {
    fn encode_snapshot(&self) -> Result<String, JsValue> {
        let digest = self
            .engine
            .authoritative_state_digest()
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let command_count = self
            .engine
            .pending_commands()
            .len()
            .saturating_add(self.engine.command_log().len());
        let snapshot = RenderSnapshot::from_world(
            self.engine.state(),
            command_count,
            self.engine.event_log().len(),
            digest,
        );
        serde_json::to_string(&snapshot).map_err(|error| JsValue::from_str(&error.to_string()))
    }
}

fn parse_seed_hex(seed_hex: &str) -> Result<u64, String> {
    if seed_hex.len() != 16 || !seed_hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("seed must be exactly 16 hexadecimal digits".to_owned());
    }
    u64::from_str_radix(seed_hex, 16).map_err(|_| "seed is outside u64 range".to_owned())
}

#[cfg(test)]
mod tests {
    use simulation_core::SimulationEngine;
    use simulation_save::{create_bundle, SaveKind};
    use simulation_worldgen::generate_trial_terrain;

    use super::parse_seed_hex;

    #[test]
    fn seed_parser_preserves_full_u64_range() {
        assert_eq!(parse_seed_hex("0000000000000000"), Ok(0));
        assert_eq!(parse_seed_hex("ffffffffffffffff"), Ok(u64::MAX));
        assert!(parse_seed_hex("1234").is_err());
        assert!(parse_seed_hex("gggggggggggggggg").is_err());
    }

    #[test]
    fn generated_terrain_can_become_authoritative_engine_state() {
        let seed = 2026;
        let mut snapshot = SimulationEngine::new(seed).export_snapshot();
        snapshot.world.terrain = Some(generate_trial_terrain(seed).expect("terrain generates"));
        let bundle = create_bundle(&snapshot, SaveKind::Manual).expect("snapshot saves");
        let restored = SimulationEngine::from_save_bundle(&bundle).expect("engine restores");
        assert!(restored.state().terrain.is_some());
        assert_eq!(restored.state().seed, seed);
    }
}
