import React from "react";
import { DataCollectionConfig, PortalRunConfig } from "../../config/config";
import { Chart } from "./Chart";
import { SimChartDataObservable } from "../../charts/SimChartDataChannel";

type ResultsViewProps = {
  config: PortalRunConfig;
  dataObservable: SimChartDataObservable;
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
