import { ChartDataset } from "chart.js";
import fp from "lodash/fp";
import { SimDataPoint, SimResultsFragments } from "../simulations/simTypes";

export type SimChartDataPoint = {
  linear: SimDataPoint;
  log2: SimDataPoint;
};

export type SimChartDataPoints = SimChartDataPoint[];

export type SimChartDataset = {
  data: SimChartDataPoints;
} & ChartDataset<"line">;

export type SimChartDatasets = SimChartDataset[];

export const emptyDatasetForReplicate = (
  replicate: number
): SimChartDataset => {
  return {
    label: `Replicate ${replicate}`,
    data: [],
    normalized: true,
  };
};

export const mergeFragmentsIntoDatasetsInPlace = (
  datasets: SimChartDatasets,
  fragments: SimResultsFragments
) => {
  for (const fragment of fragments) {
    const { replicate, points } = fragment;
    const idx = replicate - 1;
    datasets[idx] ??= emptyDatasetForReplicate(replicate);
    datasets[idx].data.push(
      ...points.map((point) => ({
        linear: point,
        log2: fp.mapValues(Math.log2)(point) as SimDataPoint,
      }))
    );
  }

  return datasets;
};
