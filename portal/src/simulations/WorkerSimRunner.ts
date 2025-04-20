import { JsSimulationHandler } from "../../build/adapter_pkg";

import { PortalRunConfig } from "../config/PortalRunConfig";
import { SimWorkerCtx } from "./workerInterface";

const yieldToEventLoop = () => new Promise((res) => setTimeout(res, 0));

/**
 * WorkerSimRunner is responsible for running the WASM code inside the web worker and reporting back results
 *
 * This is the main logic on the worker side
 */
export class WorkerSimRunner {
  static readonly #minimumPostInterval = 100;

  readonly #ctx: SimWorkerCtx;
  readonly #handler: JsSimulationHandler;
  #paused = false;

  constructor(ctx: SimWorkerCtx, config: PortalRunConfig) {
    // Data resolution property required for WASM side, but it may be missing
    //
    // This destructuring and restructuring scheme is used to clone the object,
    // but is also necessary for convincing TypeScript that the value won't be
    // undefined
    const adaptedConfig = {
      ...config,
      dataConfig: {
        ...config.dataConfig,
        dataResolution:
          config.dataConfig.dataResolution ??
          Math.max(1, Math.floor(config.simParams.transfers / 100)),
      },
    };

    this.#ctx = ctx;
    this.#handler = JsSimulationHandler.new(adaptedConfig);
  }

  pause() {
    this.#paused = true;
  }

  resume() {
    this.#paused = false;
    // We don't actually need to do anything after running, it can be thought
    // of as a continuous background process as far as this function is concerned
    this.run().then(() => {});
  }

  async run() {
    while (!this.#paused) {
      const results = this.#handler.next_fragment(
        WorkerSimRunner.#minimumPostInterval
      );

      if (results) {
        this.#ctx.postMessage({ type: "results", results });
      } else {
        this.#ctx.postMessage({
          type: "done",
          outcome: { csvDownloadUrl: this.#handler.into_output_object_url() },
        });
        return;
      }

      // Receive any messages (e.g. pause)
      await yieldToEventLoop();
    }
  }
}
