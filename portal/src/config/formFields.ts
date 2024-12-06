import { FieldPath } from "react-hook-form";

import { PortalRunConfig } from "./PortalRunConfig";

// TODO: Tooltips
export type FormFieldSet = {
  label: string;
  configPath: FieldPath<PortalRunConfig>;
  placeholder?: string;
}[];

export const simParamsFormFields: FormFieldSet = [
  {
    label: "Replicate Populations",
    configPath: "simParams.replicates",
  },
  {
    label: "Number of Transfers",
    configPath: "simParams.transfers",
  },
  {
    label: "Maximum Population Size",
    configPath: "simParams.maxPopSize",
  },
  {
    label: "Dilution Factor",
    configPath: "simParams.dilutionFactor",
  },
  {
    label: "Rate of Beneficial Mutations",
    configPath: "simParams.beneficialMutationRate",
  },
  {
    label: "Average Beneficial Mutation Effect Size",
    configPath: "simParams.initialBeneficialMutationSize",
  },
];

export const advSimParamFormFields: FormFieldSet = [
  {
    label: "Rate of Neutral Mutations",
    configPath: "simParams.neutralMutationRate",
  },
  {
    label: "Rate of Deleterious Mutations",
    configPath: "simParams.deleteriousMutationRate",
  },
  {
    label: "Strength of Epistasis",
    configPath: "simParams.diminishingReturnsEpistasisStrength",
  },
  {
    label: "Number of Initial Markers",
    configPath: "simParams.markers",
  },
  {
    label: "Randomization Seed",
    placeholder: "Automatic Seed",
    configPath: "simParams.seed",
  },
];

export const enableCSVFormField: FormFieldSet[0] = {
  label: "CSV Download",
  configPath: "dataConfig.prepareCsv",
};

export const dataResolutionFormField: FormFieldSet[0] = {
  label: "Data Resolution",
  configPath: "dataConfig.dataResolution",
  placeholder: "Automatic Resolution",
};

export const trackedStatisticsFormFields: FormFieldSet = [
  {
    label: "Average Fitness",
    configPath: "dataConfig.trackedStatistics.avgW",
  },
  {
    label: "Average Accumulated Mutations",
    configPath: "dataConfig.trackedStatistics.meanAccumulatedMuts",
  },
  {
    label: "Fitness Standard Deviation",
    configPath: "dataConfig.trackedStatistics.stdevW",
  },
  {
    label: "Shannon Genetic Diversity",
    configPath: "dataConfig.trackedStatistics.shannonDiversity",
  },
  {
    label: "Number of Genotypes",
    configPath: "dataConfig.trackedStatistics.genotypeCount",
  },
  {
    label: "Marker 1 Ratio (log2)",
    configPath: "dataConfig.trackedStatistics.marker1Ratio",
  },
  // {
  //   label: "Fitness Max",
  //   configPath: "dataConfig.trackedStatistics.maxW",
  // },
  // {
  //   label: "Accumulated Mutations Stdev",
  //   configPath: "dataConfig.trackedStatistics.stdevAccumulatedMuts",
  // },
  // {
  //   label: "Accumulated Mutations Max",
  //   configPath: "dataConfig.trackedStatistics.maxAccumulatedMuts",
  // },
  // {
  //   label: "Accumulated Mutations Min",
  //   configPath: "dataConfig.trackedStatistics.minAccumulatedMuts",
  // },
];

export const dataCollectionFormFields: FormFieldSet = [
  enableCSVFormField,
  ...trackedStatisticsFormFields,
  dataResolutionFormField,
];
