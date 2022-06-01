import { SimResultsFragment, SimWorkerHandle } from "./workerInterface";
import { PortalRunConfig } from "../config/config";

export type SimWorkerLinkConfig = {
  onResults?: (results: SimResultsFragment[]) => void;
  onFinish?: () => void;
};

export class SimWorkerLink {
  #onResults: SimWorkerLinkConfig["onResults"];
  #onFinish: SimWorkerLinkConfig["onFinish"];

  #workerHandle: SimWorkerHandle | null = null;

  constructor(config: SimWorkerLinkConfig = {}) {
    this.#onResults = config.onResults;
    this.#onFinish = config.onFinish;
  }

  start(config: PortalRunConfig) {
    this.terminate();
    this.#workerHandle = new Worker(
      new URL("./worker.ts", import.meta.url)
    ) as unknown as SimWorkerHandle;

    this.#workerHandle!.onmessage = (message) => {
      const data = message.data;

      switch (data.type) {
        case "ready":
          this.#workerHandle?.postMessage({
            type: "start",
            config: config,
          });
          return;
        case "results":
          this.#onResults?.(data.results);
          return;
        case "done":
          this.#onFinish?.();
          return;
      }
    };
  }

  pause() {
    this.#workerHandle?.postMessage({ type: "pause" });
  }

  resume() {
    this.#workerHandle?.postMessage({ type: "resume" });
  }

  terminate() {
    this.#workerHandle?.terminate();
  }
}
