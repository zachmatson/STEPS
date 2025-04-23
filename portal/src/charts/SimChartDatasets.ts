import { ChartDataset } from "chart.js";
import fp from "lodash/fp";
import { SimDataPoint, SimResultsFragments } from "../simulations/simTypes";
import { PortalRunConfig } from "../config/PortalRunConfig";
import { transfersToGenerations } from "../simulations/simUtils";

export type SimChartDataPoint = SimDataPoint & { generation: number };

export type ScaledSimChartDataPoint = {
  linear: SimChartDataPoint;
  "log2-transform": SimChartDataPoint;
};

export type ScaledSimChartDataPoints = ScaledSimChartDataPoint[];

export type SimChartDataset = {
  data: ScaledSimChartDataPoints;
} & ChartDataset<"line">;

export type SimChartDatasets = SimChartDataset[];

const emptyDatasetForReplicate = (replicate: number): SimChartDataset => {
  return {
    label: `Replicate ${replicate}`,
    data: [],
    normalized: true,
  };
};

const simDataPointToChartDataPoint = (
  point: SimDataPoint,
  config: PortalRunConfig
): ScaledSimChartDataPoint => {
  const chartDataPoint = {
    ...point,
    generation: transfersToGenerations(point.transfer, config.simParams),
  };
  return {
    linear: chartDataPoint,
    "log2-transform": fp.mapValues(Math.log2)(
      chartDataPoint
    ) as SimChartDataPoint,
  };
};

export const mergeFragmentsIntoDatasetsInPlace = (
  datasets: SimChartDatasets,
  fragments: SimResultsFragments,
  config: PortalRunConfig
) => {
  for (const fragment of fragments) {
    const { replicate, points } = fragment;
    const idx = replicate - 1;
    datasets[idx] ??= emptyDatasetForReplicate(replicate);
    datasets[idx].data.push(
      ...points.map((point) => simDataPointToChartDataPoint(point, config))
    );
  }

  return datasets;
};
