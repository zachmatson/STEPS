import fp from "lodash/fp";

import { JsSimulationHandler } from "../../steps_adapter/pkg";

import { PortalRunConfig } from "../config/PortalRunConfig";
import { SimWorkerCtx } from "./workerInterface";

const yieldToEventLoop = () => new Promise((res) => setTimeout(res, 0));

export class WorkerSimRunner {
  static readonly #minimumPostInterval = 100;

  readonly #ctx: SimWorkerCtx;
  readonly #handler: JsSimulationHandler;
  #paused = false;

  constructor(ctx: SimWorkerCtx, config: PortalRunConfig) {
    // Data resolution property required for WASM side
    config = fp.cloneDeep(config);
    config.dataConfig.dataResolution ??= Math.max(
      1,
      Math.floor(config.simParams.transfers / 100)
    );

    this.#ctx = ctx;
    this.#handler = JsSimulationHandler.new(config);
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
          downloadUrl: this.#handler.into_output_object_url(),
        });
        return;
      }

      // Receive any messages (e.g. pause)
      await yieldToEventLoop();
    }
  }
}
