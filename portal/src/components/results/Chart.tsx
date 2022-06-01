import React from "react";

import fp from "lodash/fp";
import {
  Chart as ChartJSChart,
  LinearScale,
  LineController,
  PointElement,
  LineElement,
  Tooltip,
  Title,
} from "chart.js";

ChartJSChart.register(
  LinearScale,
  LineController,
  PointElement,
  LineElement,
  Tooltip,
  Title
);

import { DataCollectionConfig, PortalRunConfig } from "../../config/config";
import { transfersToGenerations } from "../../simulations/transfersToGenerations";
import { trackedStatisticsFormFields } from "../form/formFields";
import { Observable, Subscription } from "rxjs";
import { SimChartDatasets } from "../../charts/SimChartDatasets";

const axisNameMap = Object.fromEntries(
  trackedStatisticsFormFields.map(({ label, configPath }) => [
    fp.last(configPath.split(".")),
    label,
  ])
);

export type ChartProps = {
  config: PortalRunConfig;
  stat: keyof DataCollectionConfig["trackedStatistics"];
  dataObservable: Observable<SimChartDatasets>;
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
    this.refreshChart();
    this.subscribeToData();
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
    const generations = transfersToGenerations(simParams.transfers, simParams);

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
              text: "Generation",
            },
            min: 0,
            max: generations,
            ticks: {
              includeBounds: false,
            },
          },
          y: {
            type: "linear",
            title: {
              display: true,
              text: axisNameMap[stat],
            },
          },
        },
        parsing: {
          xAxisKey: "generation",
          yAxisKey: stat,
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
