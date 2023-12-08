import { AxisScale, XAxisUnits } from "../config/chartConfig";

export const mapValueToAxisScale = (value: number, scale: AxisScale) => {
  switch (scale) {
    case "linear":
      return value;
    case "log2":
      return Math.log2(value);
  }
};

export const adaptNameForAxisScale = (name: string, scale: AxisScale) => {
  switch (scale) {
    case "linear":
      return name;
    case "log2":
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

export const xAxisKeyForScaleAndUnit = (scale: AxisScale, unit: XAxisUnits) => {
  const base = `${scale}.`;
  switch (unit) {
    case "transfers":
      return base + "transfer";
    case "generations":
      return base + "generation";
  }
};

export const yAxisKeyForScaleAndStat = (scale: AxisScale, stat: string) => {
  return `${scale}.${stat}`;
};
