import React from "react";
import { DataCollectionConfig, PortalRunConfig } from "../../config/config";
import { Chart } from "./Chart";
import { BehaviorSubject } from "rxjs";
import { SimChartDatasets } from "../../charts/SimChartDatasets";

type ResultsViewProps = {
  config: PortalRunConfig;
  dataObservable: BehaviorSubject<SimChartDatasets>;
};

export const ResultsView = ({ config, dataObservable }: ResultsViewProps) => {
  return (
    <>
      {/* TODO: Collapsible and sortable */}
      {/* TODO: Axis options */}
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
