import { readFileSync } from "node:fs";

const model = readFileSync("crates/simulation-model/src/lib.rs", "utf8");
const core = readFileSync("crates/simulation-core/src/lib.rs", "utf8");
const doc = readFileSync("docs/STAGE_1_3_COMMAND_SYSTEM.md", "utf8");

const requiredModelFragments = [
  "pub struct CommandId(pub u64)",
  "pub enum CommandSource",
  "UserIntervention = 10",
  "CountryAi = 20",
  "pub enum CommandTiming",
  "Immediate",
  "Scheduled(SimulationDate)",
  "pub struct CommandRequest",
  "pub struct QueuedCommand",
  "pub struct CommandExecutionRecord",
  "pub enum CommandError",
];

const requiredCoreFragments = [
  "pending_commands: Vec<QueuedCommand>",
  "executed_commands: Vec<CommandExecutionRecord>",
  "pub fn submit_command",
  "pub fn submit_user_command",
  "pub fn submit_country_ai_command",
  "partition_point(|command| command.execute_on <= current_date)",
  "(queued.execute_on, queued.priority, queued.id)",
  "CommandError::InvalidScheduledDate",
  "CommandError::ScheduledInPast",
];

for (const fragment of requiredModelFragments) {
  if (!model.includes(fragment)) {
    throw new Error(`Stage 1.3 model contract missing: ${fragment}`);
  }
}

for (const fragment of requiredCoreFragments) {
  if (!core.includes(fragment)) {
    throw new Error(`Stage 1.3 core contract missing: ${fragment}`);
  }
}

if (!doc.includes("(execution date, priority lane, command id)")) {
  throw new Error("Stage 1.3 documentation does not state deterministic ordering key");
}

console.log("Stage 1.3 command contract verified");
