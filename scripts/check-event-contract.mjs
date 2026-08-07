import { readFileSync } from "node:fs";

const model = readFileSync("crates/simulation-model/src/event.rs", "utf8");
const core = readFileSync("crates/simulation-core/src/lib.rs", "utf8");
const ledger = readFileSync("crates/simulation-core/src/event_ledger.rs", "utf8");
const doc = readFileSync("docs/STAGE_1_4_EVENT_LEDGER.md", "utf8");

const requiredCategories = [
  "System",
  "Economic",
  "Policy",
  "Diplomatic",
  "Military",
  "War",
  "Occupation",
  "Treaty",
  "UserIntervention",
];

const requiredModelFragments = [
  "pub struct EventId(pub u64)",
  "pub enum EventCategory",
  "pub enum EventSource",
  "pub enum EventPayload",
  "pub struct EventRecord",
  "pub struct EventFilter",
  "pub fn matches(self, event: &EventRecord) -> bool",
];

const requiredCoreFragments = [
  "event_ledger: EventLedger",
  "pub fn event_log(&self) -> &[EventRecord]",
  "pub fn query_events(&self, filter: EventFilter)",
  "EventPayload::CommandExecuted { command_id }",
  "EventCategory::UserIntervention",
  "EventSource::Country(country_id)",
  "events_emitted",
];

for (const fragment of requiredModelFragments) {
  if (!model.includes(fragment)) {
    throw new Error(`Stage 1.4 model contract missing: ${fragment}`);
  }
}

for (const category of requiredCategories) {
  if (!model.includes(category)) {
    throw new Error(`Stage 1.4 event category missing: ${category}`);
  }
}

for (const fragment of requiredCoreFragments) {
  if (!core.includes(fragment)) {
    throw new Error(`Stage 1.4 core contract missing: ${fragment}`);
  }
}

if (!ledger.includes("records: Vec<EventRecord>") || !ledger.includes("self.records.push(EventRecord")) {
  throw new Error("Stage 1.4 append-only ledger storage contract missing");
}

if (!doc.includes("Command = intent") || !doc.includes("Event = fact")) {
  throw new Error("Stage 1.4 documentation must preserve command/event separation");
}

console.log("Stage 1.4 Event Ledger contract verified");

await import("./check-save-contract.mjs");
