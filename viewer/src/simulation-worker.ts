import initWasm, { SimulationWasm } from "./generated/wasm/simulation_wasm";
import type { RenderSnapshot } from "./types";
import type { WorkerRequest, WorkerResponse } from "./worker-protocol";

let wasmReady: Promise<unknown> | undefined;
let engine: SimulationWasm | undefined;

function ensureWasm(): Promise<unknown> {
  wasmReady ??= initWasm();
  return wasmReady;
}

function parseSnapshot(snapshotJson: string): RenderSnapshot {
  return JSON.parse(snapshotJson) as RenderSnapshot;
}

function post(response: WorkerResponse): void {
  self.postMessage(response);
}

self.addEventListener("message", async (event: MessageEvent<WorkerRequest>) => {
  const request = event.data;
  try {
    await ensureWasm();

    let snapshot: RenderSnapshot;
    switch (request.type) {
      case "init":
        engine?.free();
        engine = new SimulationWasm(request.seedHex);
        snapshot = parseSnapshot(engine.snapshotJson());
        break;
      case "snapshot":
        if (engine === undefined) {
          throw new Error("simulation engine has not been initialized");
        }
        snapshot = parseSnapshot(engine.snapshotJson());
        break;
      case "advanceDays":
        if (engine === undefined) {
          throw new Error("simulation engine has not been initialized");
        }
        snapshot = parseSnapshot(engine.advanceDays(request.days));
        break;
    }

    post({ id: request.id, ok: true, snapshot });
  } catch (error) {
    post({
      id: request.id,
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    });
  }
});
