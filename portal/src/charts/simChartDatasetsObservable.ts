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
import { toSubscribedSubject } from "../utils/rxjsUtils";
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
      concat(of<SimResultsFragments>([]), sources.results$).pipe(
        scan<SimResultsFragments, SimChartDatasets>(
          mergeFragmentsIntoDatasetsInPlace,
          []
        )
      )
    ),
    toSubscribedSubject(() => new BehaviorSubject<SimChartDatasets>([])),
    observeOn(asyncScheduler)
  );
