import {
  asyncScheduler,
  BehaviorSubject,
  concat,
  Observable,
  observeOn,
  of,
  scan,
  switchMap,
} from "rxjs";

import { SimResultsFragments } from "../simulations/workerInterface";
import { makeHotImmediate } from "../utils/rxjs";
import {
  mergeFragmentsIntoDatasetsInPlace,
  SimChartDatasets,
} from "./SimChartDatasets";

export interface DatasetsSourceObservables {
  start$: Observable<unknown>;
  results$: Observable<SimResultsFragments>;
}

export const simChartDatasetsObservable = (
  sources: DatasetsSourceObservables
): Observable<SimChartDatasets> =>
  sources.start$.pipe(
    switchMap(() =>
      concat(of([] as SimResultsFragments), sources.results$).pipe(
        scan<SimResultsFragments, SimChartDatasets>(
          mergeFragmentsIntoDatasetsInPlace,
          []
        )
      )
    ),
    makeHotImmediate(() => new BehaviorSubject([] as SimChartDatasets)),
    observeOn(asyncScheduler)
  );
