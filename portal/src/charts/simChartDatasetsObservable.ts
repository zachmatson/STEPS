import { asyncScheduler, Observable, observeOn, scan, shareReplay } from "rxjs";

import {
  mergeFragmentsIntoDatasetsInPlace,
  SimChartDatasets,
} from "./SimChartDatasets";
import { SimEvent } from "../simulations/SimLink";

/**
 * Converts stream of simulation events into a stream of chart datasets
 */
export const simChartDatasetsObservable = (
  source: Observable<SimEvent>
): Observable<SimChartDatasets> =>
  source.pipe(
    scan<SimEvent, SimChartDatasets>((datasets, event) => {
      switch (event.type) {
        case "started":
          // Reset data when a new simulation run starts
          return [];
        case "results":
          // Merge results into existing datasets for the current run
          return mergeFragmentsIntoDatasetsInPlace(datasets, event.results);
        default:
          // Ignore other event types
          return datasets;
      }
    }, []),
    // Make sure all charts get latest state of the data when they subscribe
    shareReplay(1),
    // Perform chart updates asynchronously
    observeOn(asyncScheduler)
  );
