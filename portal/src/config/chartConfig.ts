import { DataCollectionConfig } from "./PortalRunConfig";

export type AxisScale = "linear" | "log2";

export type XAxisUnits = "generations" | "transfers";

export type ChartScales = {
  xScale: AxisScale;
  xUnits: XAxisUnits;
  yScale: AxisScale;
};

export type ChartScaleConfig = {
  [k in keyof DataCollectionConfig["trackedStatistics"]]: ChartScales;
};

export const defaultChartScaleConfig: ChartScaleConfig = {
  avgW: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "linear",
  },
  stdevW: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "linear",
  },
  maxW: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "linear",
  },
  marker1Ratio: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "log2",
  },
  stdevAccumulatedMuts: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "linear",
  },
  maxAccumulatedMuts: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "linear",
  },
  meanAccumulatedMuts: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "linear",
  },
  minAccumulatedMuts: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "linear",
  },
  genotypeCount: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "linear",
  },
  shannonDiversity: {
    xScale: "linear",
    xUnits: "generations",
    yScale: "linear",
  },
};
