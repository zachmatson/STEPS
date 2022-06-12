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
import { Chart } from "./Chart";
import { SimChartDatasets } from "../../charts/SimChartDatasets";
import { ChartSettingsMenu } from "./ChartSettingsMenu";

type ResultsViewProps = {
  config: PortalRunConfig;
  dataObservable: Observable<SimChartDatasets>;
};

export const ResultsView = ({ config, dataObservable }: ResultsViewProps) => {
  // TODO: Move this state up into the app
  const [scaleConfig, setScaleConfig] = useState(
    fp.cloneDeep(defaultChartScaleConfig)
  );

  return (
    <>
      {/* TODO: Sortable */}
      {Object.entries(config.dataConfig.trackedStatistics)
        .filter(([_, enabled]) => enabled)
        .map(([statUntyped, _]) => {
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
                stat={stat as keyof DataCollectionConfig["trackedStatistics"]}
                key={stat}
                config={config}
                dataObservable={dataObservable}
                scales={scaleConfig[stat]}
              />
            </Collapsible>
          );
        })}
    </>
  );
};
