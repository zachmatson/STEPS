import { z } from "zod";

import {
  zBoolean,
  zGe1Number,
  zNonNegNumber,
  zOptional,
  zPosInt,
  zPosNumber,
  zSeed,
  zString,
} from "../utils/zodUtils";

export type SimStatus = "notStarted" | "running" | "paused" | "finished";

/*
  For all sections of the simulation config, we need both a numerical schema, which is used by the simulations,
  and a "stringy" schema which losslessly stores the actual user input
 */

export const simParamsSchema = z.object({
  replicates: zPosInt,
  transfers: zPosInt,
  maxPopSize: zPosNumber,
  dilutionFactor: zGe1Number,
  markers: zPosInt,
  beneficialMutationRate: zNonNegNumber,
  neutralMutationRate: zNonNegNumber,
  deleteriousMutationRate: zNonNegNumber,
  initialBeneficialMutationSize: zPosNumber,
  fixedDeleteriousMutationSize: zOptional(zNonNegNumber),
  diminishingReturnsEpistasisStrength: zNonNegNumber,
  seed: zSeed,
});

export type SimParams = z.infer<typeof simParamsSchema>;

export const simParamsStringySchema = z.object({
  replicates: zString,
  transfers: zString,
  maxPopSize: zString,
  dilutionFactor: zString,
  markers: zString,
  beneficialMutationRate: zString,
  neutralMutationRate: zString,
  deleteriousMutationRate: zString,
  initialBeneficialMutationSize: zString,
  fixedDeleteriousMutationSize: zOptional(zString),
  diminishingReturnsEpistasisStrength: zString,
  seed: zOptional(zString),
});

export type SimParamsStringy = z.infer<typeof simParamsStringySchema>;

export const defaultSimParams: SimParamsStringy = {
  replicates: "12",
  transfers: "300",
  maxPopSize: "5e8",
  dilutionFactor: "100",
  markers: "1",
  beneficialMutationRate: "1.7e-6",
  deleteriousMutationRate: "0",
  neutralMutationRate: "0",
  initialBeneficialMutationSize: "0.012",
  fixedDeleteriousMutationSize: undefined,
  diminishingReturnsEpistasisStrength: "6.0",
  seed: undefined,
};

export const dataCollectionConfigSchema = z.object({
  prepareCsv: zBoolean,
  trackedStatistics: z.object({
    avgW: zBoolean,
    stdevW: zBoolean,
    maxW: zBoolean,
    marker1Ratio: zBoolean,
    stdevAccumulatedMuts: zBoolean,
    maxAccumulatedMuts: zBoolean,
    meanAccumulatedMuts: zBoolean,
    minAccumulatedMuts: zBoolean,
    genotypeCount: zBoolean,
    shannonDiversity: zBoolean,
  }),
  dataResolution: zOptional(zPosInt),
});

export type DataCollectionConfig = z.infer<typeof dataCollectionConfigSchema>;

export const dataCollectionConfigStringySchema = z.object({
  prepareCsv: zBoolean,
  trackedStatistics: z.object({
    avgW: zBoolean,
    stdevW: zBoolean,
    maxW: zBoolean,
    marker1Ratio: zBoolean,
    stdevAccumulatedMuts: zBoolean,
    maxAccumulatedMuts: zBoolean,
    meanAccumulatedMuts: zBoolean,
    minAccumulatedMuts: zBoolean,
    genotypeCount: zBoolean,
    shannonDiversity: zBoolean,
  }),
  dataResolution: zOptional(zString),
});

export type DataCollectionConfigStringy = z.infer<
  typeof dataCollectionConfigStringySchema
>;

export const defaultDataCollectionConfig: DataCollectionConfigStringy = {
  prepareCsv: false,
  trackedStatistics: {
    avgW: true,
    stdevW: false,
    maxW: false,
    marker1Ratio: false,
    stdevAccumulatedMuts: false,
    maxAccumulatedMuts: false,
    meanAccumulatedMuts: true,
    minAccumulatedMuts: false,
    genotypeCount: false,
    shannonDiversity: false,
  },
  dataResolution: undefined,
};

export const portalRunConfigSchema = z.object({
  simParams: simParamsSchema,
  dataConfig: dataCollectionConfigSchema,
});

export type PortalRunConfig = z.infer<typeof portalRunConfigSchema>;

export const portalRunConfigStringySchema = z.object({
  simParams: simParamsStringySchema,
  dataConfig: dataCollectionConfigStringySchema,
});

export type PortalRunConfigStringy = z.infer<
  typeof portalRunConfigStringySchema
>;

export const defaultPortalRunConfig: PortalRunConfigStringy = {
  simParams: defaultSimParams,
  dataConfig: defaultDataCollectionConfig,
};
