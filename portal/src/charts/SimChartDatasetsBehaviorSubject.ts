import { BehaviorSubject } from "rxjs";
import { SimResultsFragment } from "../simulations/workerInterface";
import {
  mergeFragmentsIntoDatasetsInPlace,
  SimChartDatasets,
} from "./SimChartDatasets";

export class SimChartDatasetsBehaviorSubject extends BehaviorSubject<SimChartDatasets> {
  #last;

  constructor(initial: SimChartDatasets) {
    super(initial);
    this.#last = initial;
  }

  next(value: SimChartDatasets) {
    this.#last = value;
    super.next(value);
  }

  nextFromFragments(fragments: SimResultsFragment[]) {
    const data = this.#last;
    mergeFragmentsIntoDatasetsInPlace(fragments, data);
    this.next(data);
  }

  resetData() {
    this.next([]);
  }
}
