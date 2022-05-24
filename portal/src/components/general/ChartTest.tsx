import React, { useRef, useEffect } from "react";

import {
  Chart,
  LinearScale,
  LineController,
  PointElement,
  LineElement,
  Tooltip,
  Title,
} from "chart.js";

Chart.register(
  LinearScale,
  LineController,
  PointElement,
  LineElement,
  Tooltip,
  Title
);

export const ChartTest = () => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const chartRef = useRef<Chart | null>(null);

  useEffect(() => {
    chartRef.current = canvasRef.current && initChart(canvasRef.current);
    console.log("Initialized chart");
  }, [canvasRef.current]);

  return <canvas ref={canvasRef} />;
};

const initChart = (canvas: HTMLCanvasElement) =>
  new Chart(canvas, {
    type: "line",
    data: {
      datasets: [],
    },
    options: {
      animation: false,
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
          max: 5000,
          ticks: {
            includeBounds: false,
          },
        },
        y: {
          type: "linear",
          title: {
            display: true,
            text: "Fitness",
          },
        },
      },
    },
  });
