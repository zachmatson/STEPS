import React, { useCallback } from "react";

import fp from "lodash/fp";
import { encodeConfigInURL } from "../config/configEncoding";

import { Button } from "./general/Button";
import { CopyToClipboardButton } from "./general/CopyToClipboardButton";
import { Tooltip } from "./general/Tooltip";
import * as icons from "./icons/Icons";
import {
  SimStatus,
  PortalRunConfigStringy,
  PortalRunConfig,
} from "../config/PortalRunConfig";

export type ButtonFooterProps = {
  status: SimStatus;
  configs: {
    numeric: PortalRunConfig;
    stringy: PortalRunConfigStringy;
  };
  configIsDirty: boolean;
  csvDownloadUrl?: string;
  startOrRestartSim: () => void;
  pauseSim: () => void;
  resumeSim: () => void;
};

export const ButtonFooter = ({
  status,
  configs,
  configIsDirty,
  csvDownloadUrl,
  startOrRestartSim,
  pauseSim,
  resumeSim,
}: ButtonFooterProps) => {
  const started = status != "notStarted";
  const running = status == "running";
  const paused = status == "paused";

  const copySeedGetText = useCallback(() => {
    const seedConfig = fp.cloneDeep(configs.stringy);
    seedConfig.simParams.seed = configs.numeric.simParams.seed.toString();
    return encodeConfigInURL(seedConfig);
  }, [configs]);
  const copyNoSeedGetText = useCallback(() => {
    const seedlessConfig = fp.cloneDeep(configs.stringy);
    seedlessConfig.simParams.seed = "";
    return encodeConfigInURL(seedlessConfig);
  }, [configs]);

  return (
    <div className="flex justify-between flex-wrap gap-4 p-2 lg:py-4 border-gray-300 border-t-2">
      <div className="flex items-center gap-1">
        <Button
          color="green"
          infoText={configIsDirty && started ? "Settings Changed" : undefined}
          onClick={startOrRestartSim}
        >
          {running || paused ? "Restart" : "Run"}
        </Button>
        {running && (
          <Button color="gray" onClick={pauseSim}>
            Pause
          </Button>
        )}
        {paused && (
          <Button color="gray" onClick={resumeSim}>
            Resume
          </Button>
        )}
      </div>

      <div className="flex items-center flex-wrap gap-4">
        <Tooltip
          text={
            !!csvDownloadUrl
              ? "Download CSV"
              : configs.numeric.dataConfig.prepareCsv
              ? "CSV download available after simulations complete"
              : "CSV download not enabled"
          }
        >
          <a href={csvDownloadUrl} download="steps_results.csv">
            <button disabled={!csvDownloadUrl}>
              <icons.Download className="h-10" />
            </button>
          </a>
        </Tooltip>
        <CopyToClipboardButton
          tooltipDescription="Copy link with seed"
          getText={copySeedGetText}
          disabled={!started}
        >
          <icons.LinkSeed className="h-10" />
        </CopyToClipboardButton>
        <CopyToClipboardButton
          tooltipDescription="Copy link without seed"
          getText={copyNoSeedGetText}
          disabled={!started}
        >
          <icons.LinkNoSeed className="h-10" />
        </CopyToClipboardButton>
      </div>
    </div>
  );
};
