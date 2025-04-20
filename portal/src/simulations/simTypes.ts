import { DataCollectionConfig } from "../config/PortalRunConfig";

export type SimDataPoint = {
  transfer: number;
} & {
  [key in keyof DataCollectionConfig["trackedStatistics"]]?: number;
};

export type SimDataPoints = SimDataPoint[];

export type SimResultsFragment = {
  replicate: number;
  points: SimDataPoints;
};

export type SimResultsFragments = SimResultsFragment[];

export type SimOutcome = {
  csvDownloadUrl?: string;
};
