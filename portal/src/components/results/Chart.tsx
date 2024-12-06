import React from "react";

import {
  Chart as ChartJSChart,
  ChartType,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
  Title,
  Tooltip,
} from "chart.js";
import { Observable, Subscription } from "rxjs";

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

ChartJSChart.register(
  LinearScale,
  LineController,
  PointElement,
  LineElement,
  Tooltip,
  Title
);

export type ChartProps = {
  config: PortalRunConfig;
  stat: keyof DataCollectionConfig["trackedStatistics"];
  dataObservable: Observable<SimChartDatasets>;
  scales: ChartScales;
};

export class Chart extends React.PureComponent<ChartProps> {
  ref = React.createRef<HTMLCanvasElement>();
  chart: ChartJSChart | undefined;
  dataSubscription: Subscription | undefined;

  render() {
    return (
      <div className="px-10 h-[22rem]">
        <canvas ref={this.ref} />
      </div>
    );
  }

  componentDidMount() {
    this.componentDidUpdate();
  }

  componentWillUnmount() {
    this.destroyChart();
    this.unsubscribeFromData();
  }

  componentDidUpdate() {
    this.refreshChart();
    this.subscribeToData();
  }

  refreshChart() {
    this.destroyChart();

    const {
      config: { simParams },
      stat,
    } = this.props;
    const { transfers } = simParams;
    const generations = transfersToGenerations(transfers, simParams);

    // TODO: Custom tooltip
    this.chart?.destroy();
    this.chart = new ChartJSChart(this.ref.current!, {
      type: "line",
      data: {
        datasets: [],
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
            type: "linear",
            title: {
              display: true,
              text: adaptNameForAxisScale(
                axisNameForXAxisUnits(this.props.scales.xUnits),
                this.props.scales.xScale
              ),
            },
            min: 0,
            max: mapValueToAxisScale(
              selectForXAxisUnits(
                transfers,
                Math.ceil(generations),
                this.props.scales.xUnits
              ),
              this.props.scales.xScale
            ),
            ticks: {
              includeBounds: true,
            },
          },
          y: {
            type: "linear",
            title: {
              display: true,
              text: adaptNameForAxisScale(
                statFormattedNames[stat],
                this.props.scales.yScale
              ),
            },
          },
        },
        parsing: {
          xAxisKey: xAxisKeyForScaleAndUnit(
            this.props.scales.xScale,
            this.props.scales.xUnits
          ),
          yAxisKey: yAxisKeyForScaleAndStat(this.props.scales.yScale, stat),
        },
      },
    });
  }

  destroyChart() {
    this.chart?.destroy();
    this.chart = undefined;
  }

  subscribeToData() {
    this.unsubscribeFromData();

    this.dataSubscription = this.props.dataObservable.subscribe((datasets) => {
      if (this.chart) {
        this.chart.data.datasets = datasets;
        this.chart.update();
      }
    });
  }

  unsubscribeFromData() {
    this.dataSubscription?.unsubscribe();
    this.dataSubscription = undefined;
  }
}

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
