import { z } from "zod";

import {
  zPosNumber,
  zPosInt,
  zString,
  zBoolean,
  zSeed,
  zNonNegNumber,
  zGe1Number,
} from "../utils/zodUtils";
import { Base64 } from "js-base64";
import { safeJSONParse, SafeParseResult } from "../utils/safeJSONParse";

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
  deleteriousMutationSizeFactor: zPosNumber(),
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
  deleteriousMutationSizeFactor: zString,
  mutationRateMutationSizeFactor: zString,
  diminishingReturnsEpistasisStrength: zString,
  seed: zString.default(""),
});

export type SimParamsStringy = z.infer<typeof simParamsStringySchema>;

export const defaultSimParams: SimParamsStringy = {
  replicates: "4",
  transfers: "800",
  maxPopSize: "5e8",
  dilutionFactor: "100",
  markers: "2",
  beneficialMutationRate: "1.7e-6",
  deleteriousMutationRate: "0",
  mutationRateMutationRate: "0",
  neutralMutationRate: "0",
  initialBeneficialMutationSize: "0.01587",
  deleteriousMutationSizeFactor: "1",
  mutationRateMutationSizeFactor: "1",
  diminishingReturnsEpistasisStrength: "6.0217",
  seed: "",
};

export const dataCollectionConfigSchema = z.object({
  prepareCSV: zBoolean,
  trackedStatistics: z.object({
    avgW: zBoolean,
    marker1Ratio: zBoolean,
    stdevW: zBoolean,
    maxW: zBoolean,
    stdevAccumulatedMuts: zBoolean,
    maxAccumulatedMuts: zBoolean,
    genotypeCount: zBoolean,
    shannonDiversity: zBoolean,
  }),
  dataResolution: zPosInt({ optional: true }),
});

export type DataCollectionConfig = z.infer<typeof dataCollectionConfigSchema>;

export const dataCollectionConfigStringySchema = z.object({
  prepareCSV: zBoolean,
  trackedStatistics: z.object({
    avgW: zBoolean,
    marker1Ratio: zBoolean,
    stdevW: zBoolean,
    maxW: zBoolean,
    stdevAccumulatedMuts: zBoolean,
    maxAccumulatedMuts: zBoolean,
    genotypeCount: zBoolean,
    shannonDiversity: zBoolean,
  }),
  dataResolution: zString.default(""),
});

export type DataCollectionConfigStringy = z.infer<
  typeof dataCollectionConfigStringySchema
>;

export const defaultDataCollectionConfig: DataCollectionConfigStringy = {
  prepareCSV: false,
  trackedStatistics: {
    avgW: true,
    stdevW: false,
    maxW: false,
    marker1Ratio: false,
    stdevAccumulatedMuts: false,
    maxAccumulatedMuts: false,
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

export const encodeConfigInURL = (
  config: PortalRunConfig | PortalRunConfigStringy,
  seed: boolean,
  baseHref = window.location.href
): string => {
  const url = new URL(baseHref);
  const stringyConfig = portalRunConfigStringySchema.parse(config);
  if (!seed) {
    stringyConfig.simParams.seed = "";
  }

  url.searchParams.set(
    "runConfig",
    Base64.encode(JSON.stringify(stringyConfig))
  );
  return url.toString();
};

export const decodeConfigFromURL = (
  href = window.location.href
): SafeParseResult<PortalRunConfigStringy> => {
  const url = new URL(href);
  const encodedConfig = url.searchParams.get("runConfig");
  if (!encodedConfig) return { success: false };
  const decoded = safeJSONParse(Base64.decode(encodedConfig));
  if (!decoded.success) return decoded;
  const checked = portalRunConfigStringySchema.safeParse(decoded.data);
  if (!checked.success) return checked;
  return { success: true, data: checked.data };
};
