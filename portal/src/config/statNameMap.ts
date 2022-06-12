import fp from "lodash/fp";
import { DataCollectionConfig } from "./PortalRunConfig";

import { trackedStatisticsFormFields } from "./formFields";

export const statFormattedNames: Record<
  keyof DataCollectionConfig["trackedStatistics"],
  string
> = Object.fromEntries(
  trackedStatisticsFormFields.map(({ label, configPath }) => [
    fp.last(configPath.split(".")),
    label,
  ])
);
