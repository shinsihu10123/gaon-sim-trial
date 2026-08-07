import { readFileSync } from "node:fs";

const core = readFileSync("crates/simulation-core/src/event_ledger.rs", "utf8");
const doc = readFileSync("docs/STAGE_1_6_DETERMINISM.md", "utf8");

const requiredCoreFragments = [
  "pub fn authoritative_state_digest(&self) -> Result<u64, SaveError>",
  "checksum_fnv1a64(&bundle.state_binary)",
  "pub fn accepted_command_journal(&self) -> Vec<QueuedCommand>",
  "pub fn replay_from_journal(",
  "pub fn deterministic_random_u64(&self, stream: u64, ordinal: u64) -> u64",
  "counter_random_u64(self.state.seed, self.state.elapsed_days, stream, ordinal)",
  "journal.sort_by_key(|command| command.id)",
];

for (const fragment of requiredCoreFragments) {
  if (!core.includes(fragment)) {
    throw new Error(`Stage 1.6 determinism contract missing: ${fragment}`);
  }
}

for (const phrase of [
  "authoritative state digest",
  "counter-based random",
  "command journal replay",
  "10,000-tick replay",
]) {
  if (!doc.includes(phrase)) {
    throw new Error(`Stage 1.6 documentation missing requirement: ${phrase}`);
  }
}

console.log("Stage 1.6 determinism contract verified");
