import { readFileSync } from "node:fs";

const save = readFileSync("crates/simulation-save/src/lib.rs", "utf8");
const binary = readFileSync("crates/simulation-save/src/binary.rs", "utf8");
const coreLedger = readFileSync("crates/simulation-core/src/event_ledger.rs", "utf8");
const doc = readFileSync("docs/STAGE_1_5_SAVE_RESTORE.md", "utf8");

const saveFragments = [
  "pub const SAVE_FORMAT_VERSION: u32 = 1",
  "pub enum SaveKind",
  "Manual",
  "Autosave",
  "pub struct SaveMetadata",
  "pub struct EngineSnapshot",
  "pub struct SaveBundle",
  "pub struct AutosavePolicy",
  "pub fn create_bundle",
  "pub fn decode_bundle",
  "pub fn validate_snapshot",
  "ChecksumMismatch",
  "UnsupportedMetadataVersion",
  "UnsupportedBinaryVersion",
  "SnapshotInvariant",
];

const binaryFragments = [
  'const STATE_MAGIC: [u8; 8] = *b"GAONST01"',
  "to_le_bytes()",
  "from_le_bytes",
  "pending_commands",
  "executed_commands",
  "events",
  "next_command_id",
  "next_event_id",
  "TrailingBinaryData",
  "TruncatedBinary",
];

const coreFragments = [
  "pub fn export_snapshot(&self) -> EngineSnapshot",
  "pub fn save_bundle(&self, kind: SaveKind)",
  "pub fn from_save_bundle(bundle: &SaveBundle)",
  "EventLedger::from_snapshot",
];

for (const fragment of saveFragments) {
  if (!save.includes(fragment)) {
    throw new Error(`Stage 1.5 save contract missing: ${fragment}`);
  }
}

for (const fragment of binaryFragments) {
  if (!binary.includes(fragment)) {
    throw new Error(`Stage 1.5 binary contract missing: ${fragment}`);
  }
}

for (const fragment of coreFragments) {
  if (!coreLedger.includes(fragment)) {
    throw new Error(`Stage 1.5 core restore contract missing: ${fragment}`);
  }
}

if (!doc.includes("metadata.json") || !doc.includes("state.bin")) {
  throw new Error("Stage 1.5 documentation must define JSON metadata + binary state");
}

if (!doc.includes("100-year continuation")) {
  throw new Error("Stage 1.5 documentation must require long-run continuation validation");
}

console.log("Stage 1.5 save/restore contract verified");

await import("./check-determinism-contract.mjs");
