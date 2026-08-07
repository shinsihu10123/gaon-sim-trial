import type { RenderSnapshot } from "./types";

export type WorkerRequest =
  | { id: number; type: "init"; seedHex: string }
  | { id: number; type: "snapshot" }
  | { id: number; type: "advanceDays"; days: number };

export type WorkerResponse =
  | { id: number; ok: true; snapshot: RenderSnapshot }
  | { id: number; ok: false; error: string };
