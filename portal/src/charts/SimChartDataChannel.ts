import {
  mergeFragmentsIntoDatasetsInPlace,
  SimChartDatasets,
} from "./SimChartDatasets";
import { SimResultsFragment } from "../simulations/workerInterface";

export type SimChartDataSubscription = {
  unsubscribe: () => void;
};

export type SimChartDataSubscriptionOnData = (data: SimChartDatasets) => void;

export type SimChartDataSubscriptionCallbacks = {
  onData: SimChartDataSubscriptionOnData;
};

export type SimChartDataObservable = {
  subscribe: (
    callbacks: SimChartDataSubscriptionCallbacks
  ) => SimChartDataSubscription;
};

export class SimChartDataChannel {
  #data: SimChartDatasets = [];
  #subscribers: SubscriptionInternal[] = [];

  #subscribe: SimChartDataObservable["subscribe"] = (callbacks) => {
    const newSubInternal: SubscriptionInternal = {
      ...callbacks,
      subscribed: true,
    };
    this.#subscribers.push(newSubInternal);
    newSubInternal.onData(this.#data);

    return {
      unsubscribe: () => {
        const internalIndex = this.#subscribers.indexOf(newSubInternal);
        if (internalIndex) {
          this.#subscribers[internalIndex].subscribed = false;
          this.#subscribers = this.#subscribers.splice(internalIndex, 1);
        }
      },
    };
  };

  #observable: SimChartDataObservable = {
    subscribe: this.#subscribe,
  };

  observable() {
    return this.#observable;
  }

  pushFragments(fragments: SimResultsFragment[]) {
    mergeFragmentsIntoDatasetsInPlace(fragments, this.#data);
    this.#notifyAllWithCurrentData();
  }

  clear() {
    this.#data = [];
    this.#notifyAllWithCurrentData();
  }

  #notifyAllWithCurrentData() {
    const currentData = this.#data;
    this.#subscribers.forEach((sub) => {
      if (sub.subscribed) {
        setTimeout(() => sub.onData(currentData));
      }
    });
  }
}

type SubscriptionInternal = {
  onData: (data: SimChartDatasets) => void;
  subscribed: boolean;
};
