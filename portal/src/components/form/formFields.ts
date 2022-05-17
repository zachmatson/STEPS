import { FieldPath } from "react-hook-form";

import { PortalRunConfig } from "../../config/config";

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
