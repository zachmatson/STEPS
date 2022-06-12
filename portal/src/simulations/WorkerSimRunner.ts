import fp from "lodash/fp";

import { JSSimulationHandler } from "../../steps_adapter/pkg";

import {
  DataCollectionConfig,
  PortalRunConfig,
} from "../config/PortalRunConfig";
import { SimResultsFragments, SimWorkerCtx } from "./workerInterface";
import { transfersToGenerations } from "./transfersToGenerations";

const statNameMap = <const>{
  avgW: "avg_W",
  marker1Ratio: "marker_1_ratio",
  stdevW: "stdev_W",
  maxW: "max_W",
  stdevAccumulatedMuts: "stdev_accumulated_muts",
  maxAccumulatedMuts: "max_accumulated_muts",
  genotypeCount: "genotype_count",
  shannonDiversity: "shannon_diversity",
};

const paramNameMap = <const>{
  replicates: "replicates",
  transfers: "transfers",
  maxPopSize: "max_pop_size",
  dilutionFactor: "dilution_factor",
  markers: "markers",
  beneficialMutationRate: "beneficial_mutation_rate",
  neutralMutationRate: "neutral_mutation_rate",
  deleteriousMutationRate: "deleterious_mutation_rate",
  mutationRateMutationRate: "mutation_rate_mutation_rate",
  initialBeneficialMutationSize: "initial_beneficial_mutation_size",
  deleteriousMutationSizeFactor: "deleterious_mutation_size_factor",
  mutationRateMutationSizeFactor: "mutation_rate_mutation_size_factor",
  diminishingReturnsEpistasisStrength: "diminishing_returns_epistasis_strength",
  seed: "seed",
};

const extractSimParams = (config: PortalRunConfig) =>
  fp.mapKeys((key: string) => paramNameMap[key as keyof typeof paramNameMap])(
    config.simParams
  );

const yieldToEventLoop = () => new Promise((res) => setTimeout(res, 0));

export class WorkerSimRunner {
  readonly #ctx: SimWorkerCtx;

  readonly #config: PortalRunConfig;
  readonly #handler: JSSimulationHandler;
  readonly #enabledStats: (keyof DataCollectionConfig["trackedStatistics"])[];
  readonly #resolution: number;

  #replicate = 1;
  #transfer = 0;

  #paused = false;

  #buffer: SimResultsFragments = [];
  #lastPostTime = Date.now();
  readonly #minimumPostInterval = 100;

  constructor(ctx: SimWorkerCtx, config: PortalRunConfig) {
    this.#ctx = ctx;
    this.#config = config;
    this.#handler = JSSimulationHandler.new(extractSimParams(config));
    this.#enabledStats = Object.entries(config.dataConfig.trackedStatistics)
      .filter(([_, val]) => val)
      .map(
        ([key, _]) => key as keyof DataCollectionConfig["trackedStatistics"]
      );
    this.#resolution =
      config.dataConfig.dataResolution ??
      Math.max(1, Math.floor(config.simParams.transfers / 100));
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
    const replicates = this.#config.simParams.replicates;
    const transfers = this.#config.simParams.transfers;

    while (!this.#paused) {
      if (this.#transfer == 0) {
        this.#handler.start_replicate();
        await this.#record();
      }

      const remainingTransfers = transfers - this.#transfer;

      if (remainingTransfers > this.#resolution) {
        await this.#advanceAndRecord(this.#resolution);
      } else {
        await this.#advanceAndRecord(remainingTransfers);

        if (this.#replicate < replicates) {
          this.#replicate += 1;
          this.#transfer = 0;
        } else {
          await this.#finalize();
          break;
        }
      }
    }
  }

  async #advanceAndRecord(advanceBy: number) {
    this.#handler.advance(BigInt(advanceBy));
    this.#transfer += advanceBy;
    await this.#record();
  }

  async #record() {
    // Results fragments should be from single replicate only
    if (
      this.#buffer.length == 0 ||
      fp.last(this.#buffer)!.replicate != this.#replicate
    ) {
      this.#buffer.push({
        replicate: this.#replicate,
        points: [],
      });
    }

    fp.last(this.#buffer)!.points.push({
      generation: transfersToGenerations(
        this.#transfer,
        this.#config.simParams
      ),
      ...Object.fromEntries(
        this.#enabledStats.map((stat) => [
          stat,
          this.#handler[`check_${statNameMap[stat]}`](),
        ])
      ),
    });

    await this.#flushAndReceiveData({ force: false });
  }

  async #finalize() {
    await this.#flushAndReceiveData({ force: true });
    this.#ctx.postMessage({ type: "done" });
  }

  async #flushAndReceiveData({ force }: { force: boolean }) {
    const now = Date.now();
    if (
      (force || now - this.#lastPostTime > this.#minimumPostInterval) &&
      this.#buffer
    ) {
      this.#ctx.postMessage({ type: "results", results: this.#buffer });
      this.#lastPostTime = now;
      this.#buffer = [];

      // Allow us to receive a potential message to pause
      await yieldToEventLoop();
    }
  }
}
