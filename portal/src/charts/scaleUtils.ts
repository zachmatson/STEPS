import { AxisScale } from "../config/chartConfig";

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
