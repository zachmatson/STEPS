import { SimParams } from "../config/PortalRunConfig";

export const transfersToGenerations = (
  transfers: number,
  simParams: SimParams
): number => transfers * Math.log2(simParams.dilutionFactor);
