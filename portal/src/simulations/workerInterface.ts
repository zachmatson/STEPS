import { DataCollectionConfig, PortalRunConfig } from "../config/config";

export type SimDataPoint = {
  generation: number;
} & {
  [key in keyof DataCollectionConfig["trackedStatistics"]]?: number;
};

export type SimDataPoints = SimDataPoint[];

export type SimResultsFragment = {
  replicate: number;
  points: SimDataPoints;
};

export type SimResultsFragments = SimResultsFragment[];

export type InboundSimWorkerMessage =
  | { type: "start"; config: PortalRunConfig }
  | { type: "pause" }
  | { type: "resume" };

export type OutboundSimWorkerMessageReady = { type: "ready" };
export type OutboundSimWorkerMessageResults = {
  type: "results";
  results: SimResultsFragments;
};
export type OutboundSimWorkerMessageDone = { type: "done" };

export type OutboundSimWorkerMessage =
  | OutboundSimWorkerMessageReady
  | OutboundSimWorkerMessageResults
  | OutboundSimWorkerMessageDone;

export interface SimWorkerHandle
  extends Omit<Worker, "postMessage" | "onmessage"> {
  onmessage: (message: { data: OutboundSimWorkerMessage }) => void;
  postMessage: (message: InboundSimWorkerMessage) => void;
}

export interface SimWorkerCtx {
  onmessage: (message: { data: InboundSimWorkerMessage }) => void;
  postMessage: (message: OutboundSimWorkerMessage) => void;
}
