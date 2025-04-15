import { z } from "zod";

import {
  zBoolean,
  zGe1Number,
  zNonNegNumber,
  zPosInt,
  zPosNumber,
  zSeed,
  zString,
} from "../utils/zodUtils";

export type SimStatus = "notStarted" | "running" | "paused" | "finished";

export const simParamsSchema = z.object({
  replicates: zPosInt(),
  transfers: zPosInt(),
  maxPopSize: zPosNumber(),
  dilutionFactor: zGe1Number(),
  markers: zPosInt(),
  beneficialMutationRate: zNonNegNumber(),
  neutralMutationRate: zNonNegNumber(),
  deleteriousMutationRate: zNonNegNumber(),
  mutationRateMutationRate: zNonNegNumber(),
  initialBeneficialMutationSize: zPosNumber(),
  fixedDeleteriousMutationSize: zNonNegNumber(),
  mutationRateMutationSizeFactor: zPosNumber(),
  diminishingReturnsEpistasisStrength: zNonNegNumber(),
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
  mutationRateMutationRate: zString,
  initialBeneficialMutationSize: zString,
  fixedDeleteriousMutationSize: zString,
  mutationRateMutationSizeFactor: zString,
  diminishingReturnsEpistasisStrength: zString,
  seed: zString.default(""),
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
  mutationRateMutationRate: "0",
  neutralMutationRate: "0",
  initialBeneficialMutationSize: "0.012",
  fixedDeleteriousMutationSize: "2.0",
  mutationRateMutationSizeFactor: "1",
  diminishingReturnsEpistasisStrength: "6.0",
  seed: "",
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
  // TODO: See if we can use fancier type signatures in the underlying methods instead of this
  dataResolution: zPosInt({
    optional: true,
  }) as unknown as z.ZodOptional<z.ZodNumber>,
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
  dataResolution: zString.default(""),
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
    meanAccumulatedMuts: false,
    minAccumulatedMuts: false,
    genotypeCount: false,
    shannonDiversity: false,
  },
  dataResolution: "",
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
