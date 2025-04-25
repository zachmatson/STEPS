import React, { useRef, memo, useCallback, useEffect } from "react";

import {
  Chart as ChartJSChart,
  ChartType,
  LinearScale,
  LineController,
  LineElement,
  LogarithmicScale,
  PointElement,
  Title,
  Tooltip,
} from "chart.js";
import { Observable } from "rxjs";

import {
  adaptNameForAxisScale,
  axisNameForXAxisUnits,
  mapValueToAxisScale,
  selectForXAxisUnits,
  xAxisKeyForScaleAndUnit,
  yAxisKeyForScaleAndStat,
} from "../../charts/scaleUtils";
import { SimChartDatasets } from "../../charts/SimChartDatasets";
import { ChartScales } from "../../config/chartConfig";
import {
  DataCollectionConfig,
  PortalRunConfig,
} from "../../config/PortalRunConfig";
import { statFormattedNames } from "../../config/statNameMap";
import { transfersToGenerations } from "../../simulations/simUtils";
import { useSubscription } from "../../utils/reactUtils";

ChartJSChart.register(
  LinearScale,
  LogarithmicScale,
  LineController,
  PointElement,
  LineElement,
  Tooltip,
  Title
);

export type ChartProps = {
  config: PortalRunConfig;
  stat: keyof DataCollectionConfig["trackedStatistics"];
  scales: ChartScales;
  dataObservable: Observable<SimChartDatasets>;
};

export const Chart = memo(function Chart(props: ChartProps) {
  const { config, stat, scales, dataObservable } = props;

  const { canvasRef, updateData } = useChart(config, stat, scales);
  useSubscription(dataObservable, updateData);

  return (
    <div className="px-10 h-[22rem]">
      <canvas ref={canvasRef} />
    </div>
  );
});

const useChart = (
  config: PortalRunConfig,
  stat: keyof DataCollectionConfig["trackedStatistics"],
  scales: ChartScales
) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  // Use a separate ref for data so that we can pass it back in when we replace the chart
  const dataRef = useRef<SimChartDatasets>([]);
  const chartRef = useRef<ChartJSChart | null>(null);

  const updateData = useCallback((data: SimChartDatasets) => {
    dataRef.current = data;
    if (chartRef.current) {
      chartRef.current.data.datasets = data;
      chartRef.current.update();
    }
  }, []);

  useEffect(() => {
    if (canvasRef.current) {
      chartRef.current = createChart(
        canvasRef.current,
        config,
        stat,
        scales,
        dataRef.current
      );
    }

    // Destroy chart on cleanup
    return () => {
      chartRef.current?.destroy();
      chartRef.current = null;
    };
  }, [config, scales, stat]);

  return { canvasRef, updateData };
};

const createChart = (
  canvas: HTMLCanvasElement,
  config: PortalRunConfig,
  stat: keyof DataCollectionConfig["trackedStatistics"],
  scales: ChartScales,
  initialData: SimChartDatasets
): ChartJSChart => {
  const { simParams } = config;
  const { transfers } = simParams;
  const generations = transfersToGenerations(transfers, simParams);

  return new ChartJSChart(canvas, {
    type: "line",
    data: {
      datasets: initialData,
    },
    plugins: [
      {
        id: "colorScheme",
        beforeDatasetUpdate(
          chart: ChartJSChart<ChartType>,
          args: { index: number }
        ) {
          const dataset = chart.data.datasets[args.index];
          const color = COLOR_SCHEME[args.index % COLOR_SCHEME.length];
          dataset.borderColor = color;
          dataset.backgroundColor = color;
        },
      },
    ],
    options: {
      animation: false,
      maintainAspectRatio: false,
      plugins: {
        tooltip: {
          mode: "index",
          position: "average",
          intersect: false,
          backgroundColor: "rgba(0, 0, 0, 0.5)",
        },
      },
      hover: {
        intersect: false,
        mode: "index",
      },
      elements: {
        point: {
          radius: 0,
          hoverRadius: 4,
        },
      },
      scales: {
        x: {
          type: scales.xScale === "log" ? "logarithmic" : "linear",
          title: {
            display: true,
            text: adaptNameForAxisScale(
              axisNameForXAxisUnits(scales.xUnits),
              scales.xScale
            ),
          },
          min: 0,
          max: mapValueToAxisScale(
            selectForXAxisUnits(
              transfers,
              Math.ceil(generations),
              scales.xUnits
            ),
            scales.xScale
          ),
          ticks: {
            includeBounds: true,
          },
        },
        y: {
          type: scales.yScale === "log" ? "logarithmic" : "linear",
          title: {
            display: true,
            text: adaptNameForAxisScale(
              statFormattedNames[stat],
              scales.yScale
            ),
          },
        },
      },
      parsing: {
        xAxisKey: xAxisKeyForScaleAndUnit(scales.xScale, scales.xUnits),
        yAxisKey: yAxisKeyForScaleAndStat(scales.yScale, stat),
      },
    },
  });
};

// Colorblind friendly scheme from
// https://github.com/nagix/chartjs-plugin-colorschemes/blob/d96a01846626881aa4bec56828c333af81050906/src/colorschemes/colorschemes.tableau.js#L8
const COLOR_SCHEME = [
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
