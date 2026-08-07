import { readFileSync } from "node:fs";

const model = readFileSync("crates/simulation-model/src/lib.rs", "utf8");
const core = readFileSync("crates/simulation-core/src/lib.rs", "utf8");
const doc = readFileSync("docs/STAGE_1_2_TIME_SYSTEM.md", "utf8");

const modelFragments = [
  "pub struct SimulationDate",
  "pub const START: Self = Self",
  "year: 1",
  "month: 1",
  "day: 1",
  "const MONTH_LENGTHS: [u8; 12]",
  "pub fn advance_one_day(&mut self) -> DateBoundary",
  "month_changed",
  "quarter_changed",
  "year_changed",
];

const coreFragments = [
  "pub enum SimulationSpeed",
  "Paused",
  "X1",
  "X7",
  "X30",
  "X90",
  "X365",
  "pub fn tick(&mut self) -> TickReport",
  "pub fn step_one_day(&mut self) -> AdvanceReport",
  "pub fn step_one_month(&mut self) -> AdvanceReport",
  "pub const fn render_stride_days(self) -> u32",
  "scaled_nanosecond_remainder: u128",
  "elapsed.as_nanos()",
];

for (const fragment of modelFragments) {
  if (!model.includes(fragment)) {
    throw new Error(`Stage 1.2 calendar contract missing: ${fragment}`);
  }
}

for (const fragment of coreFragments) {
  if (!core.includes(fragment)) {
    throw new Error(`Stage 1.2 core time contract missing: ${fragment}`);
  }
}

for (const phrase of [
  "1 simulation tick = 1 simulated day",
  "Logic/render separation",
  "365x",
  "integer nanosecond",
]) {
  if (!doc.includes(phrase)) {
    throw new Error(`Stage 1.2 documentation missing requirement: ${phrase}`);
  }
}

console.log("Stage 1.2 time/tick contract verified");
