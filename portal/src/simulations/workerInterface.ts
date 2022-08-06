import { PortalRunConfig } from "../config/PortalRunConfig";
import { SimOutcome, SimResultsFragments } from "./simTypes";

export type SimWorkerInboundMessage =
  | { type: "start"; config: PortalRunConfig }
  | { type: "pause" }
  | { type: "resume" };

export type SimWorkerOutboundMessage =
  | { type: "ready" }
  | { type: "results"; results: SimResultsFragments }
  | { type: "done"; outcome: SimOutcome };

export interface SimWorkerHandle
  extends Omit<Worker, "postMessage" | "onmessage"> {
  onmessage: (message: { data: SimWorkerOutboundMessage }) => void;
  postMessage: (message: SimWorkerInboundMessage) => void;
}

export interface SimWorkerCtx {
  onmessage: (message: { data: SimWorkerInboundMessage }) => void;
  postMessage: (message: SimWorkerOutboundMessage) => void;
}
