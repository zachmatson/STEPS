import { SimParams } from "../config/PortalRunConfig";

export const transfersToGenerations = (
  transfers: number,
  simParams: SimParams
): number => {
  const dilution = simParams.dilutionFactor;
  if (dilution != 100) {
    return transfers * Math.log2(simParams.dilutionFactor);
  } else {
    return (transfers * 20.0) / 3.0;
  }
};
