import { ChartDataset } from "chart.js";
import fp from "lodash/fp";

import {
  SimDataPoint,
  SimResultsFragments,
} from "../simulations/workerInterface";

export type SimChartDataPoint = {
  linear: SimDataPoint;
  log2: SimDataPoint;
};

export type SimChartDataPoints = SimChartDataPoint[];

export type SimChartDataset = {
  data: SimChartDataPoints;
} & ChartDataset<"line">;

export type SimChartDatasets = SimChartDataset[];

// Colorblind friendly scheme from
// https://github.com/nagix/chartjs-plugin-colorschemes/blob/d96a01846626881aa4bec56828c333af81050906/src/colorschemes/colorschemes.tableau.js#L8
const colors = [
  "#1170aa",
  "#fc7d0b",
  "#a3acb9",
  "#57606c",
  "#5fa2ce",
  "#c85200",
  "#7b848f",
  "#a3cce9",
  "#ffbc79",
  "#c8d0d9",
];

export const emptyDatasetForReplicate = (
  replicate: number
): SimChartDataset => {
  const colorIdx = (replicate - 1) % colors.length;
  return {
    label: `Replicate ${replicate}`,
    data: [],
    borderColor: colors[colorIdx],
    backgroundColor: colors[colorIdx],
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
