import type { RenderSnapshot, RenderTerrainSnapshot } from "./types";
import type { WorkerRequest, WorkerRequestPayload, WorkerResponse } from "./worker-protocol";

type PendingRequest = {
  resolve: (snapshot: RenderSnapshot) => void;
  reject: (error: Error) => void;
};

export class SimulationBridge {
  private readonly worker: Worker;
  private readonly pending = new Map<number, PendingRequest>();
  private nextRequestId = 1;
  private staticTerrain: RenderTerrainSnapshot | null = null;

  public constructor() {
    this.worker = new Worker(new URL("./simulation-worker.ts", import.meta.url), {
      type: "module",
      name: "gaon-simulation-worker",
    });
    this.worker.addEventListener("message", (event: MessageEvent<WorkerResponse>) => {
      this.handleResponse(event.data);
    });
    this.worker.addEventListener("error", (event) => {
      const error = new Error(event.message || "simulation worker failed");
      for (const request of this.pending.values()) {
        request.reject(error);
      }
      this.pending.clear();
    });
  }

  public initialize(seedHex: string): Promise<RenderSnapshot> {
    this.staticTerrain = null;
    return this.request({ type: "init", seedHex });
  }

  public snapshot(): Promise<RenderSnapshot> {
    return this.request({ type: "snapshot" });
  }

  public advanceDays(days: number): Promise<RenderSnapshot> {
    if (!Number.isInteger(days) || days < 0 || days > 0xffff_ffff) {
      return Promise.reject(new Error("days must be a non-negative u32 integer"));
    }
    return this.request({ type: "advanceDays", days });
  }

  public dispose(): void {
    this.worker.terminate();
    const error = new Error("simulation bridge disposed");
    for (const request of this.pending.values()) {
      request.reject(error);
    }
    this.pending.clear();
    this.staticTerrain = null;
  }

  private request(payload: WorkerRequestPayload): Promise<RenderSnapshot> {
    const id = this.nextRequestId;
    this.nextRequestId += 1;
    const message: WorkerRequest = { id, ...payload };

    return new Promise<RenderSnapshot>((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.worker.postMessage(message);
    });
  }

  private handleResponse(response: WorkerResponse): void {
    const request = this.pending.get(response.id);
    if (request === undefined) {
      return;
    }
    this.pending.delete(response.id);
    if (!response.ok) {
      request.reject(new Error(response.error));
      return;
    }

    const snapshot = response.snapshot;
    if (snapshot.world.terrain !== null) {
      this.staticTerrain = snapshot.world.terrain;
      request.resolve(snapshot);
      return;
    }
    if (this.staticTerrain === null && snapshot.world.initialized) {
      request.reject(new Error("dynamic snapshot arrived before static terrain initialization"));
      return;
    }

    request.resolve({
      ...snapshot,
      world: {
        ...snapshot.world,
        terrain: this.staticTerrain,
      },
    });
  }
}
