import React from "react";

import { Observable } from "rxjs";

import { DataCollectionConfig, PortalRunConfig } from "../../config/config";
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
  return (
    <>
      {/* TODO: Collapsible and sortable */}
      {/* TODO: Axis options */}
      {Object.entries(config.dataConfig.trackedStatistics)
        .filter(([_, enabled]) => enabled)
        .map(([statUntyped, _]) => {
          const stat =
            statUntyped as keyof DataCollectionConfig["trackedStatistics"];
          const statName = statFormattedNames[stat];
          return (
            <Collapsible
              title={statName}
              defaultExpanded
              settingsIcon={<ChartSettingsMenu statName={statName} />}
            >
              <Chart
                stat={stat as keyof DataCollectionConfig["trackedStatistics"]}
                key={stat}
                config={config}
                dataObservable={dataObservable}
              />
            </Collapsible>
          );
        })}
    </>
  );
};
