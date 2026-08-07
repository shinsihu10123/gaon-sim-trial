import type { RenderSnapshot } from "./types";
import type { WorkerRequest, WorkerResponse } from "./worker-protocol";

type PendingRequest = {
  resolve: (snapshot: RenderSnapshot) => void;
  reject: (error: Error) => void;
};

export class SimulationBridge {
  private readonly worker: Worker;
  private readonly pending = new Map<number, PendingRequest>();
  private nextRequestId = 1;

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
  }

  private request(payload: Omit<WorkerRequest, "id">): Promise<RenderSnapshot> {
    const id = this.nextRequestId;
    this.nextRequestId += 1;
    const message = { id, ...payload } as WorkerRequest;

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
    if (response.ok) {
      request.resolve(response.snapshot);
    } else {
      request.reject(new Error(response.error));
    }
  }
}
