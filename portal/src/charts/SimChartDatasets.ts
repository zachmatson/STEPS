import { SimResultsFragment } from "../simulations/workerInterface";
import { ChartDataset } from "chart.js";

export type SimChartDataset = {
  data: SimResultsFragment["points"];
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

const emptyDatasetForReplicate = (replicate: number) => {
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
  fragments: SimResultsFragment[],
  datasets: SimChartDatasets
) => {
  for (const fragment of fragments) {
    const { replicate, points } = fragment;
    const idx = replicate - 1;
    datasets[idx] ??= emptyDatasetForReplicate(replicate);
    datasets[idx].data.push(...points);
  }

  return datasets;
};
