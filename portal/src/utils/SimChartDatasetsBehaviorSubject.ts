import { BehaviorSubject } from "rxjs";
import { SimChartDatasets } from "../components/results/Chart";
import { SimResultsFragment } from "../simulations/workerInterface";

export class SimChartDatasetsBehaviorSubject extends BehaviorSubject<SimChartDatasets> {
  #last;

  constructor(initial: SimChartDatasets) {
    super(initial);
    this.#last = initial;
  }

  next(value: SimChartDatasets) {
    this.#last = value;
    super.next(value);
  }

  nextFromFragments(fragments: SimResultsFragment[]) {
    const data = this.#last;

    for (const fragment of fragments) {
      const { replicate, points } = fragment;
      const idx = replicate - 1;
      data[idx] ??= {
        label: `Replicate ${replicate}`,
        data: [],
        borderColor: colors[idx % colors.length],
        backgroundColor: colors[idx % colors.length],
        normalized: true,
      };
      data[idx].data.push(...points);
    }

    this.next(data);
  }

  resetData() {
    this.next([]);
  }
}

// Colorblind friendly scheme from
// https://github.com/nagix/chartjs-plugin-colorschemes/blob/d96a01846626881aa4bec56828c333af81050906/src/colorschemes/colorschemes.tableau.js#L8
const colors = [
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
