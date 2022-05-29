import React from "react";
import { DataCollectionConfig, PortalRunConfig } from "../../config/config";
import { Chart, SimChartDatasets } from "./Chart";
import { BehaviorSubject } from "rxjs";

type ResultsViewProps = {
  config: PortalRunConfig;
  dataObservable: BehaviorSubject<SimChartDatasets>;
};

export const ResultsView = ({ config, dataObservable }: ResultsViewProps) => {
  return (
    <>
      {/* TODO: Collapsible and sortable */}
      {Object.entries(config.dataConfig.trackedStatistics)
        .filter(([_, enabled]) => enabled)
        .map(([stat, _]) => (
          <Chart
            stat={stat as keyof DataCollectionConfig["trackedStatistics"]}
            key={stat}
            config={config}
            dataObservable={dataObservable}
          />
        ))}
    </>
  );
};
