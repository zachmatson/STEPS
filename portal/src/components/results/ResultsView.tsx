import React, { useMemo, useState } from "react";

import fp from "lodash/fp";
import { Observable } from "rxjs";

import { defaultChartScaleConfig } from "../../config/chartConfig";
import {
  DataCollectionConfig,
  PortalRunConfig,
} from "../../config/PortalRunConfig";
import { statFormattedNames } from "../../config/statNameMap";
import { Collapsible } from "../general/Collapsible";
import { SortableMappedList } from "../general/SortableMappedList";
import { Chart } from "./Chart";
import { ChartSettingsMenu } from "./ChartSettingsMenu";
import { SimEvent } from "../../simulations/SimLink";
import { simChartDatasetsObservable } from "../../charts/simChartDatasetsObservable";

export type ResultsViewProps = {
  config: PortalRunConfig;
  simEvent$: Observable<SimEvent>;
};

export const ResultsView = ({ config, simEvent$ }: ResultsViewProps) => {
  // TODO: Move this state up into the app to save it in the URL
  const [scaleConfig, setScaleConfig] = useState(() =>
    fp.cloneDeep(defaultChartScaleConfig)
  );

  const enabledStats = fp.pipe(
    fp.pickBy(fp.identity),
    fp.keys
  )(config.dataConfig.trackedStatistics);

  const dataObservable = useMemo(
    () => simChartDatasetsObservable(simEvent$),
    [simEvent$]
  );

  return (
    <SortableMappedList keys={enabledStats} handle>
      {({ key, handleClassname }) => {
        const stat = key as keyof DataCollectionConfig["trackedStatistics"];
        const statName = statFormattedNames[stat];

        return (
          <Collapsible
            title={statName}
            key={stat}
            defaultExpanded
            dragHandleClassname={handleClassname}
            settingsIcon={
              <ChartSettingsMenu
                scalesValue={scaleConfig[stat]}
                onScalesChange={(value) =>
                  setScaleConfig((config) => ({ ...config, [stat]: value }))
                }
              />
            }
          >
            <Chart
              stat={stat}
              key={stat}
              config={config}
              dataObservable={dataObservable}
              scales={scaleConfig[stat]}
            />
          </Collapsible>
        );
      }}
    </SortableMappedList>
  );
};
