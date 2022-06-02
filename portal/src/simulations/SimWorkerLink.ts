import { SimResultsFragments } from "./workerInterface";
import { PortalRunConfig } from "../config/config";
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

  observables() {
    return this.#publicObservables;
  }
}
