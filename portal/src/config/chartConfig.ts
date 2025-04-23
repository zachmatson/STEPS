import { DataCollectionConfig } from "./PortalRunConfig";

export type AxisScale = "linear" | "log" | "log2-transform";

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
    yScale: "log2-transform",
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

export type AvailableScales = {
  yScales: readonly AxisScale[];
  xScales: readonly AxisScale[];
};

export type AvailableScalesByStat = {
  [k in keyof DataCollectionConfig["trackedStatistics"]]: AvailableScales;
};

export const availableScalesByStat: AvailableScalesByStat = {
  avgW: {
    yScales: ["linear", "log"],
    xScales: ["linear", "log"],
  },
  stdevW: {
    yScales: ["linear", "log"],
    xScales: ["linear", "log"],
  },
  maxW: {
    yScales: ["linear", "log"],
    xScales: ["linear", "log"],
  },
  marker1Ratio: {
    yScales: ["linear", "log2-transform"],
    xScales: ["linear", "log"],
  },
  stdevAccumulatedMuts: {
    yScales: ["linear", "log"],
    xScales: ["linear", "log"],
  },
  maxAccumulatedMuts: {
    yScales: ["linear", "log"],
    xScales: ["linear", "log"],
  },
  meanAccumulatedMuts: {
    yScales: ["linear", "log"],
    xScales: ["linear", "log"],
  },
  minAccumulatedMuts: {
    yScales: ["linear", "log"],
    xScales: ["linear", "log"],
  },
  genotypeCount: {
    yScales: ["linear", "log"],
    xScales: ["linear", "log"],
  },
  shannonDiversity: {
    yScales: ["linear", "log"],
    xScales: ["linear", "log"],
  },
};
