import type { RenderSnapshot } from "./types";

export type WorkerRequestPayload =
  | { type: "init"; seedHex: string }
  | { type: "snapshot" }
  | { type: "advanceDays"; days: number };

export type WorkerRequest = WorkerRequestPayload & { id: number };

export type WorkerResponse =
  | { id: number; ok: true; snapshot: RenderSnapshot }
  | { id: number; ok: false; error: string };
