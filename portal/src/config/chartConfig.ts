import { DataCollectionConfig } from "./PortalRunConfig";

export type AxisScale = "linear" | "log2";

export type ChartScales = {
  x: AxisScale;
  y: AxisScale;
};

export type ChartScaleConfig = {
  [k in keyof DataCollectionConfig["trackedStatistics"]]: ChartScales;
};

export const defaultChartScaleConfig: ChartScaleConfig = {
  avgW: {
    x: "linear",
    y: "linear",
  },
  stdevW: {
    x: "linear",
    y: "linear",
  },
  maxW: {
    x: "linear",
    y: "linear",
  },
  marker1Ratio: {
    x: "linear",
    y: "log2",
  },
  stdevAccumulatedMuts: {
    x: "linear",
    y: "linear",
  },
  maxAccumulatedMuts: {
    x: "linear",
    y: "linear",
  },
  genotypeCount: {
    x: "linear",
    y: "linear",
  },
  shannonDiversity: {
    x: "linear",
    y: "linear",
  },
};
