import { SimOutcome, SimResultsFragments } from "./simTypes";
import { SimWorkerHandle, SimWorkerOutboundMessage } from "./workerInterface";
import { PortalRunConfig } from "../config/PortalRunConfig";
import { Subject } from "rxjs";

export type SimEvent =
  | { type: "started"; config: PortalRunConfig }
  | { type: "results"; results: SimResultsFragments }
  | { type: "done"; outcome: SimOutcome };

/**
 * SimLink is responsible for serving as an interface between the simulations and the rest of the portal code,
 * abstracting away the details of the worker and WASM and exposing a stream of events.
 */
export class SimLink {
  readonly #event$ = new Subject<SimEvent>();
  #workerHandle: SimWorkerHandle | undefined;

  eventsObservable() {
    return this.#event$.asObservable();
  }

  startOrRestart(config: PortalRunConfig) {
    // Stop the old worker, no messages from the old worker will be processed any more
    this.#workerHandle?.terminate();
    // Start a fresh web worker
    this.#workerHandle = new Worker(
      new URL("./worker.ts", import.meta.url)
    ) as unknown as SimWorkerHandle;
    // Configure responses to messages from the worker
    this.#workerHandle.onmessage = ({ data }) =>
      this.#onWorkerMessage(data, config);
    // Inform listeners that a new simulation run has started
    // This is guranteed to run before we process any messages from the new worker, because we haven't yielded to the
    // event loop yet.
    this.#event$.next({ type: "started", config });
  }

  #onWorkerMessage(data: SimWorkerOutboundMessage, config: PortalRunConfig) {
    switch (data.type) {
      case "ready":
        // The worker is ready to start, so we will tell it to
        this.#workerHandle?.postMessage({ type: "start", config: config });
        break;
      case "results":
        // Forward results to listeners
        this.#event$.next({ type: "results", results: data.results });
        break;
      case "done":
        // Forward outcome to listeners
        this.#event$.next({ type: "done", outcome: data.outcome });
        break;
    }
  }

  pause() {
    this.#workerHandle?.postMessage({ type: "pause" });
  }

  resume() {
    this.#workerHandle?.postMessage({ type: "resume" });
  }
}
