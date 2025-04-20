import {
  asyncScheduler,
  map,
  Observable,
  observeOn,
  scan,
  shareReplay,
} from "rxjs";

import {
  mergeFragmentsIntoDatasetsInPlace,
  SimChartDatasets,
} from "./SimChartDatasets";
import { SimEvent } from "../simulations/SimLink";
import { PortalRunConfig } from "../config/PortalRunConfig";

/**
 * Converts stream of simulation events into a stream of chart datasets
 */
export const simChartDatasetsObservable = (
  source: Observable<SimEvent>
): Observable<SimChartDatasets> =>
  source.pipe(
    scan<SimEvent, [PortalRunConfig | null, SimChartDatasets]>(
      ([config, datasets], event) => {
        switch (event.type) {
          case "started":
            // Reset data when a new simulation run starts
            return [event.config, []];
          case "results":
            // Merge results into existing datasets for the current run
            return [
              config,
              mergeFragmentsIntoDatasetsInPlace(
                datasets,
                event.results,
                config!
              ),
            ];
          default:
            // Ignore other event types
            return [config, datasets];
        }
      },
      [null, []]
    ),
    map(([_config, datasets]) => datasets),
    // Make sure all charts get latest state of the data when they subscribe
    shareReplay(1),
    // Perform chart updates asynchronously
    observeOn(asyncScheduler)
  );
