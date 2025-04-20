import { WorkerSimRunner } from "./WorkerSimRunner";
import { SimWorkerCtx } from "./workerInterface";

/**
 * This file is the actual script that gets run when the web worker is launched
 * It serves as scaffolding between the raw worker message interface and the WorkerSimRunner
 */

const ctx = self as unknown as SimWorkerCtx;
let runner: WorkerSimRunner | null = null;

ctx.onmessage = (message) => {
  const data = message.data;

  switch (data.type) {
    case "start":
      runner = new WorkerSimRunner(ctx, data.config);
      runner.run().then(() => {});
      return;
    case "pause":
      runner?.pause();
      return;
    case "resume":
      runner?.resume();
      return;
  }
};

// We are ready to receive messages!
// Getting here means the WASM in the runner has finished loading
ctx.postMessage({ type: "ready" });
