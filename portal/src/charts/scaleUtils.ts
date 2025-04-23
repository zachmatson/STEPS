import { AxisScale, XAxisUnits } from "../config/chartConfig";

export const mapValueToAxisScale = (value: number, scale: AxisScale) => {
  switch (scale) {
    case "linear":
      return value;
    case "log":
      return value; // Native Chart.js log scale will handle this
    case "log2-transform":
      return Math.log2(value);
  }
};

export const adaptNameForAxisScale = (name: string, scale: AxisScale) => {
  switch (scale) {
    case "linear":
      return name;
    case "log":
      return name;
    case "log2-transform":
      return `${name} (log2)`;
  }
};

export const selectForXAxisUnits = (
  transfers: number,
  generations: number,
  unit: XAxisUnits
) => {
  switch (unit) {
    case "transfers":
      return transfers;
    case "generations":
      return generations;
  }
};

export const axisNameForXAxisUnits = (unit: XAxisUnits) => {
  switch (unit) {
    case "transfers":
      return "Transfers";
    case "generations":
      return "Generations";
  }
};

const scaleKeyForScale = (scale: AxisScale) =>
  // log2-transform uses the log2-transformed values
  // everything else uses linear values and relies on Chart.js for scaling
  scale === "log2-transform" ? scale : "linear";

export const xAxisKeyForScaleAndUnit = (scale: AxisScale, unit: XAxisUnits) => {
  const scaleKey = scaleKeyForScale(scale);
  const base = `${scaleKey}.`;
  switch (unit) {
    case "transfers":
      return base + "transfer";
    case "generations":
      return base + "generation";
  }
};

export const yAxisKeyForScaleAndStat = (scale: AxisScale, stat: string) => {
  const scaleKey = scaleKeyForScale(scale);
  return `${scaleKey}.${stat}`;
};
