import { readFileSync } from "node:fs";

const entity = readFileSync("crates/simulation-model/src/entity.rs", "utf8");
const model = readFileSync("crates/simulation-model/src/lib.rs", "utf8");
const lifecycle = readFileSync("crates/simulation-core/src/entity_lifecycle.rs", "utf8");
const binary = readFileSync("crates/simulation-save/src/binary.rs", "utf8");
const protocol = readFileSync("crates/simulation-protocol/src/lib.rs", "utf8");
const tests = readFileSync("crates/simulation-core/tests/dynamic_entity.rs", "utf8");

for (const fragment of [
  "typed_entity_id!(HumanGroupId)",
  "typed_entity_id!(SettlementId)",
  "typed_entity_id!(CommunityId)",
  "typed_entity_id!(PoliticalEntityId)",
  "typed_entity_id!(CountryId)",
  "typed_entity_id!(RegionId)",
  "typed_entity_id!(CityId)",
  "pub struct HumanGroupEntity",
  "pub struct HumanGroupRegistry",
  "impl EntityRegistry for HumanGroupRegistry",
  "identity_registry!(SettlementId, SettlementEntity, SettlementRegistry)",
  "identity_registry!(CommunityId, CommunityEntity, CommunityRegistry)",
  "PoliticalEntityRegistry",
  "pub human_groups: HumanGroupRegistry",
  "pub settlements: SettlementRegistry",
  "pub communities: CommunityRegistry",
  "pub political_entities: PoliticalEntityRegistry",
  "pub countries: CountryRegistry",
  "pub cities: CityRegistry",
  "BTreeMap",
  "next_id",
]) {
  if (!entity.includes(fragment)) {
    throw new Error(`Stage 2.5 entity contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "CreateHumanGroupEntity",
  "CreateSettlementEntity",
  "CreateCommunityEntity",
  "CreatePoliticalEntity",
  "PromotePoliticalEntityToCountry",
  "human_groups: HumanGroupRegistry::new()",
  "settlements: SettlementRegistry::new()",
  "communities: CommunityRegistry::new()",
  "political_entities: PoliticalEntityRegistry::new()",
  "countries: CountryRegistry::new()",
]) {
  if (!model.includes(fragment)) {
    throw new Error(`Stage 2.5 World/Command contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "CommandPayload::CreateHumanGroupEntity",
  "CommandPayload::CreateSettlementEntity",
  "CommandPayload::CreateCommunityEntity",
  "CommandPayload::CreatePoliticalEntity",
  "CommandPayload::PromotePoliticalEntityToCountry",
  "EventPayload::EntityTransitioned",
]) {
  if (!lifecycle.includes(fragment)) {
    throw new Error(`Stage 2.5 lifecycle contract missing: ${fragment}`);
  }
}

for (const fragment of [
  '"human_groups"',
  '"settlements"',
  '"communities"',
  '"political_entities"',
  "HumanGroupRegistry::from_parts",
  "SettlementRegistry::from_parts",
  "CommunityRegistry::from_parts",
  "PoliticalEntityRegistry::from_parts",
  "write_u8(bytes, 15)",
  "EventPayload::EntityTransitioned",
  "EntityKind::HumanGroup",
  "EntityKind::Settlement",
  "EntityKind::Community",
  "EntityKind::PoliticalEntity",
]) {
  if (!binary.includes(fragment)) {
    throw new Error(`Stage 2.5 save contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "pub id: String",
  "pub neighbors: Vec<String>",
  "pub legal_owner: Option<String>",
  "pub controller: Option<String>",
]) {
  if (!protocol.includes(fragment)) {
    throw new Error(`Stage 2.5 JS-safe ID contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "civilization_origin_entities_exist_before_any_country",
  "pre_state_save_load_preserves_all_allocators",
  "political_entity_can_transition_to_country_without_predefining_emergence_cause",
  "pre_state_and_country_transition_journal_replay_matches_authoritative_digest",
]) {
  if (!tests.includes(fragment)) {
    throw new Error(`Stage 2.5 acceptance coverage missing: ${fragment}`);
  }
}

for (const forbidden of ["COUNTRY_COUNT =", "const COUNTRY_COUNT"]) {
  if (model.includes(forbidden) || entity.includes(forbidden)) {
    throw new Error(`fixed Country-count invariant is forbidden: ${forbidden}`);
  }
}

console.log("Stage 2.5 civilization-origin Dynamic Entity contract verified");
