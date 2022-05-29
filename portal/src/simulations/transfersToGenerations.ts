import { SimParams } from "../config/config";

export const transfersToGenerations = (
  transfers: number,
  simParams: SimParams
): number => transfers * Math.log2(simParams.dilutionFactor);
