import { DataCollectionConfig, PortalRunConfig } from "../config/config";

export type SimResultsFragment = {
  replicate: number;
  transfer: number[];
} & {
  [key in keyof DataCollectionConfig["trackedStatistics"]]?: number[];
};

export type InboundSimWorkerMessage =
  | { type: "start"; config: PortalRunConfig }
  | { type: "pause" }
  | { type: "resume" };

export type OutboundSimWorkerMessage =
  | { type: "ready" }
  | { type: "results"; results: SimResultsFragment }
  | { type: "done" };

export interface SimWorkerHandle
  extends Omit<Worker, "postMessage" | "onmessage"> {
  onmessage: (message: { data: OutboundSimWorkerMessage }) => void;
  postMessage: (message: InboundSimWorkerMessage) => void;
}

export interface SimWorkerCtx {
  onmessage: (message: { data: InboundSimWorkerMessage }) => void;
  postMessage: (message: OutboundSimWorkerMessage) => void;
}
