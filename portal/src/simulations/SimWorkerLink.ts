import { SimResultsFragments, SimWorkerHandle } from "./workerInterface";
import { PortalRunConfig } from "../config/PortalRunConfig";
import { Observable, Subject } from "rxjs";

export interface SimObservables {
  start$: Observable<PortalRunConfig>;
  results$: Observable<SimResultsFragments>;
  done$: Observable<void>;
}

export class SimWorkerLink {
  #start$: Subject<PortalRunConfig> = new Subject();
  #results$: Subject<SimResultsFragments> = new Subject();
  #done$: Subject<void> = new Subject();
  #publicObservables: SimObservables = {
    start$: this.#start$.asObservable(),
    results$: this.#results$.asObservable(),
    done$: this.#done$.asObservable(),
  };

  #workerHandle: SimWorkerHandle | undefined;

  startOrRestart(config: PortalRunConfig) {
    this.#workerHandle?.terminate();
    this.#workerHandle = new Worker(
      new URL("./worker.ts", import.meta.url)
    ) as unknown as SimWorkerHandle;

    this.#workerHandle.onmessage = ({ data }) => {
      switch (data.type) {
        case "ready":
          this.#workerHandle?.postMessage({ type: "start", config: config });
          break;
        case "results":
          this.#results$.next(data.results);
          break;
        case "done":
          this.#done$.next();
          break;
      }
    };

    this.#start$.next(config);
  }

  pause() {
    this.#workerHandle?.postMessage({ type: "pause" });
  }

  resume() {
    this.#workerHandle?.postMessage({ type: "resume" });
  }

  observables() {
    return this.#publicObservables;
  }
}
