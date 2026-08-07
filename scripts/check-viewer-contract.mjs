import { readFileSync } from "node:fs";

const workspace = readFileSync("Cargo.toml", "utf8");
const wasm = readFileSync("crates/simulation-wasm/src/lib.rs", "utf8");
const worker = readFileSync("viewer/src/simulation-worker.ts", "utf8");
const bridge = readFileSync("viewer/src/simulation-bridge.ts", "utf8");
const viewerPackage = JSON.parse(readFileSync("viewer/package.json", "utf8"));

for (const member of ["crates/simulation-protocol", "crates/simulation-wasm"]) {
  if (!workspace.includes(member)) {
    throw new Error(`workspace member missing: ${member}`);
  }
}

for (const fragment of [
  "engine: SimulationEngine",
  "RenderSnapshot::from_world",
  "authoritative_state_digest",
]) {
  if (!wasm.includes(fragment)) {
    throw new Error(`WASM ownership contract missing: ${fragment}`);
  }
}

for (const fragment of [
  "generated/wasm/simulation_wasm",
  "let engine: SimulationWasm | undefined",
  "new SimulationWasm",
]) {
  if (!worker.includes(fragment)) {
    throw new Error(`worker ownership contract missing: ${fragment}`);
  }
}

if (!bridge.includes("new Worker(new URL")) {
  throw new Error("main-thread bridge does not use a Web Worker");
}

if (viewerPackage.dependencies?.three === undefined) {
  throw new Error("Three.js dependency missing");
}
if (
  viewerPackage.devDependencies?.vite === undefined ||
  viewerPackage.devDependencies?.typescript === undefined
) {
  throw new Error("Vite/TypeScript development dependencies missing");
}

console.log("Stage 1.1 browser ownership contract verified");
await import("./check-world-data-contract.mjs");
await import("./check-terrain-contract.mjs");
