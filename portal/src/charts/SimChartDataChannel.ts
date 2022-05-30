import {
  mergeFragmentsIntoDatasetsInPlace,
  SimChartDatasets,
} from "./SimChartDatasets";
import { SimResultsFragment } from "../simulations/workerInterface";

export type SimChartDataSubscription = {
  unsubscribe: () => void;
};

export type SubscribeToSimChartData = (
  next: (data: SimChartDatasets) => void
) => SimChartDataSubscription;

export class SimChartDataChannel {
  #data: SimChartDatasets = [];
  #subscribers: SubscriptionInternal[] = [];

  subscribable = (): SubscribeToSimChartData => (next) => {
    const newSubHandle = {
      unsubscribe: () => {},
    };

    const newSubInternal: SubscriptionInternal = {
      next,
      subscribed: true,
      externalHandle: newSubHandle,
    };

    newSubHandle.unsubscribe = () => {
      const internalIndex = this.#subscribers.findIndex(
        (subInternal) => subInternal.externalHandle === newSubHandle
      );
      if (internalIndex) {
        this.#subscribers[internalIndex].subscribed = false;
        this.#subscribers = this.#subscribers.splice(internalIndex, 1);
      }
    };

    this.#subscribers.push(newSubInternal);
    return newSubHandle;
  };

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
        setTimeout(() => sub.next(currentData));
      }
    });
  }
}

type SubscriptionInternal = {
  next: (data: SimChartDatasets) => void;
  subscribed: boolean;
  externalHandle: SimChartDataSubscription;
};
