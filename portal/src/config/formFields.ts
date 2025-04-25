import { FieldPath } from "react-hook-form";

import { DataCollectionConfig, PortalRunConfig } from "./PortalRunConfig";

// TODO: Tooltips
export type FormField = {
  label: string;
  configPath: FieldPath<PortalRunConfig>;
  placeholder?: string;
};

export type FormFieldSet = FormField[];

export type StatFormField = FormField & {
  stat: keyof DataCollectionConfig["trackedStatistics"];
};

export type StatFormFieldSet = StatFormField[];

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

export const enableCSVFormField: FormField = {
  label: "CSV Download",
  configPath: "dataConfig.prepareCsv",
};

export const dataResolutionFormField: FormField = {
  label: "Data Resolution",
  configPath: "dataConfig.dataResolution",
  placeholder: "Automatic Resolution",
};

export const trackedStatisticsFormFields: StatFormFieldSet = [
  {
    stat: "avgW",
    label: "Average Fitness",
    configPath: "dataConfig.trackedStatistics.avgW",
  },
  {
    stat: "meanAccumulatedMuts",
    label: "Average Accumulated Mutations",
    configPath: "dataConfig.trackedStatistics.meanAccumulatedMuts",
  },
  {
    stat: "stdevW",
    label: "Fitness Standard Deviation",
    configPath: "dataConfig.trackedStatistics.stdevW",
  },
  {
    stat: "shannonDiversity",
    label: "Shannon Genetic Diversity",
    configPath: "dataConfig.trackedStatistics.shannonDiversity",
  },
  {
    stat: "genotypeCount",
    label: "Number of Genotypes",
    configPath: "dataConfig.trackedStatistics.genotypeCount",
  },
  {
    stat: "marker1Ratio",
    label: "Marker 1 Ratio",
    configPath: "dataConfig.trackedStatistics.marker1Ratio",
  },
];

export const dataCollectionFormFields: FormFieldSet = [
  enableCSVFormField,
  ...trackedStatisticsFormFields,
  dataResolutionFormField,
];
