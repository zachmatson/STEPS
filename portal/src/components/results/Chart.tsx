import React from "react";

import {
  Chart as ChartJSChart,
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
                generations,
                this.props.scales.xUnits
              ),
              this.props.scales.xScale
            ),
            ticks: {
              includeBounds: false,
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
