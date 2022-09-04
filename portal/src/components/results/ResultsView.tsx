import React, { useState } from "react";

import fp from "lodash/fp";
import { Observable } from "rxjs";

import { defaultChartScaleConfig } from "../../config/chartConfig";
import {
  DataCollectionConfig,
  PortalRunConfig,
} from "../../config/PortalRunConfig";
import { statFormattedNames } from "../../config/statNameMap";
import { Collapsible } from "../general/Collapsible";
import { SortabbleMappedList } from "../general/SortableMappedList";
import { Chart } from "./Chart";
import { SimChartDatasets } from "../../charts/SimChartDatasets";
import { ChartSettingsMenu } from "./ChartSettingsMenu";

export type ResultsViewProps = {
  config: PortalRunConfig;
  dataObservable: Observable<SimChartDatasets>;
};

/*
    Idea for the API of SortableMappedList:
      Take a sorted list as input via controlled component pattern
      Take comprehensive list of elements as another input
      Use some reconciler strategy
 */

export const ResultsView = ({ config, dataObservable }: ResultsViewProps) => {
  // TODO: Move this state up into the app to save it in the URL
  const [scaleConfig, setScaleConfig] = useState(() =>
    fp.cloneDeep(defaultChartScaleConfig)
  );

  const enabledStats = fp.pipe(
    fp.pickBy(fp.identity),
    fp.keys
  )(config.dataConfig.trackedStatistics);

  return (
    <SortabbleMappedList
      keys={enabledStats}
      renderKey={(statUntyped) => {
        const stat =
          statUntyped as keyof DataCollectionConfig["trackedStatistics"];
        const statName = statFormattedNames[stat];

        return (
          <Collapsible
            title={statName}
            key={stat}
            defaultExpanded
            settingsIcon={
              <ChartSettingsMenu
                scalesValue={scaleConfig[stat]}
                onScalesChange={(value) =>
                  setScaleConfig((config) => ({ ...config, [stat]: value }))
                }
              />
            }
          >
            <Chart
              stat={stat}
              key={stat}
              config={config}
              dataObservable={dataObservable}
              scales={scaleConfig[stat]}
            />
          </Collapsible>
        );
      }}
    />
  );
};
