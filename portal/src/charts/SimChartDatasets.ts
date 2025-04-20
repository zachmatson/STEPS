import { ChartDataset } from "chart.js";
import fp from "lodash/fp";
import { SimDataPoint, SimResultsFragments } from "../simulations/simTypes";

export type ExtendedSimDataPoint = SimDataPoint & { generation: number };

export type SimChartDataPoint = {
  linear: ExtendedSimDataPoint;
  log2: ExtendedSimDataPoint;
};

export type SimChartDataPoints = SimChartDataPoint[];

export type SimChartDataset = {
  data: SimChartDataPoints;
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
  point: SimDataPoint
): SimChartDataPoint => ({
  linear: point,
  log2: fp.mapValues(Math.log2)(point) as SimDataPoint,
});

export const mergeFragmentsIntoDatasetsInPlace = (
  datasets: SimChartDatasets,
  fragments: SimResultsFragments
) => {
  for (const fragment of fragments) {
    const { replicate, points } = fragment;
    const idx = replicate - 1;
    datasets[idx] ??= emptyDatasetForReplicate(replicate);
    datasets[idx].data.push(...points.map(simDataPointToChartDataPoint));
  }

  return datasets;
};
