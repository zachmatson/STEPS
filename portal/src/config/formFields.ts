import { FieldPath } from "react-hook-form";

import { PortalRunConfig } from "./config";

// TODO: Tooltips
export type FormFieldSet = {
  label: string;
  configPath: FieldPath<PortalRunConfig>;
  placeholder?: string;
}[];

export const simParamsFormFields: FormFieldSet = [
  {
    label: "Replicates",
    configPath: "simParams.replicates",
  },
  {
    label: "Transfers",
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
    label: "Beneficial Mutation Rate",
    configPath: "simParams.beneficialMutationRate",
  },
  {
    label: "Beneficial Mutation Size",
    configPath: "simParams.initialBeneficialMutationSize",
  },
  {
    label: "Seed",
    placeholder: "Automatic Seed",
    configPath: "simParams.seed",
  },
];

export const advSimParamFormFields: FormFieldSet = [
  {
    label: "Markers",
    configPath: "simParams.markers",
  },
  {
    label: "Neutral Mutation Rate",
    configPath: "simParams.neutralMutationRate",
  },
  {
    label: "Diminishing Returns Epistasis Strength",
    configPath: "simParams.diminishingReturnsEpistasisStrength",
  },
];

export const enableCSVFormField: FormFieldSet[0] = {
  label: "CSV Download",
  configPath: "dataConfig.prepareCSV",
};

export const dataResolutionFormField: FormFieldSet[0] = {
  label: "Data Resolution",
  configPath: "dataConfig.dataResolution",
  placeholder: "Automatic Resolution",
};

export const trackedStatisticsFormFields: FormFieldSet = [
  {
    label: "Fitness Mean",
    configPath: "dataConfig.trackedStatistics.avgW",
  },
  {
    label: "Fitness Stdev",
    configPath: "dataConfig.trackedStatistics.stdevW",
  },
  {
    label: "Fitness Max",
    configPath: "dataConfig.trackedStatistics.maxW",
  },
  {
    label: "Marker 1 Ratio",
    configPath: "dataConfig.trackedStatistics.marker1Ratio",
  },
  {
    label: "Accumulated Mutations Stdev",
    configPath: "dataConfig.trackedStatistics.stdevAccumulatedMuts",
  },
  {
    label: "Accumulated Mutations Max",
    configPath: "dataConfig.trackedStatistics.maxAccumulatedMuts",
  },
  {
    label: "Genotype Count",
    configPath: "dataConfig.trackedStatistics.genotypeCount",
  },
  {
    label: "Shannon Diversity",
    configPath: "dataConfig.trackedStatistics.shannonDiversity",
  },
];

export const dataCollectionFormFields: FormFieldSet = [
  enableCSVFormField,
  ...trackedStatisticsFormFields,
  dataResolutionFormField,
];
