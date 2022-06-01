import {
  asyncScheduler,
  BehaviorSubject,
  concat,
  Observable,
  observeOn,
  of,
  scan,
  Subject,
  switchMap,
} from "rxjs";
import { SimResultsFragment } from "../simulations/workerInterface";
import {
  mergeFragmentsIntoDatasetsInPlace,
  SimChartDatasets,
} from "./SimChartDatasets";
import { connectWithSubject } from "../utils/rxjs";

export type SimChartDatasetsChannel = {
  clear: () => void;
  datasets$: Observable<SimChartDatasets>;
};

export const simChartDatasetsChannel = (
  results$: Observable<SimResultsFragment[]>
): SimChartDatasetsChannel => {
  const clear$ = new Subject<void>();
  const datasets$ = clear$.pipe(
    switchMap(() =>
      concat(of([] as SimResultsFragment[]), results$).pipe(
        scan<SimResultsFragment[], SimChartDatasets>(
          mergeFragmentsIntoDatasetsInPlace,
          []
        )
      )
    ),
    connectWithSubject(() => new BehaviorSubject([] as SimChartDatasets)),
    observeOn(asyncScheduler)
  );

  return {
    clear: () => clear$.next(),
    datasets$,
  };
};
