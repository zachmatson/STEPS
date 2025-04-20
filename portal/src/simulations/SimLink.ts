import { SimOutcome, SimResultsFragments } from "./simTypes";
import { SimWorkerHandle, SimWorkerOutboundMessage } from "./workerInterface";
import { PortalRunConfig } from "../config/PortalRunConfig";
import { Observable, Subject } from "rxjs";

export interface SimObservables {
  start$: Observable<PortalRunConfig>;
  results$: Observable<SimResultsFragments>;
  outcome$: Observable<SimOutcome>;
}

export class SimLink {
  #start$: Subject<PortalRunConfig> = new Subject();
  #results$: Subject<SimResultsFragments> = new Subject();
  #outcome$: Subject<SimOutcome> = new Subject();
  #publicObservables: SimObservables = {
    start$: this.#start$.asObservable(),
    results$: this.#results$.asObservable(),
    outcome$: this.#outcome$.asObservable(),
  };

  #workerHandle: SimWorkerHandle | undefined;

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
    this.#start$.next(config);
  }

  #onWorkerMessage(data: SimWorkerOutboundMessage, config: PortalRunConfig) {
    switch (data.type) {
      case "ready":
        this.#workerHandle?.postMessage({ type: "start", config: config });
        break;
      case "results":
        this.#results$.next(data.results);
        break;
      case "done":
        this.#outcome$.next(data.outcome);
        break;
    }
  }

  pause() {
    this.#workerHandle?.postMessage({ type: "pause" });
  }

  resume() {
    this.#workerHandle?.postMessage({ type: "resume" });
  }

  observables(): SimObservables {
    return this.#publicObservables;
  }
}
