import React, { useMemo, useState } from "react";

import fp from "lodash/fp";
import { Observable } from "rxjs";

import {
  availableScalesByStat,
  defaultChartScaleConfig,
} from "../../config/chartConfig";
import { PortalRunConfig } from "../../config/PortalRunConfig";
import { Collapsible } from "../general/Collapsible";
import { SortableMappedList } from "../general/SortableMappedList";
import { Chart } from "./Chart";
import { ChartSettingsMenu } from "./ChartSettingsMenu";
import { SimEvent } from "../../simulations/SimLink";
import { simChartDatasetsObservable } from "../../charts/simChartDatasetsObservable";
import { trackedStatisticsFormFields } from "../../config/formFields";

export type ResultsViewProps = {
  config: PortalRunConfig;
  simEvent$: Observable<SimEvent>;
};

export const ResultsView = ({ config, simEvent$ }: ResultsViewProps) => {
  // TODO: Move this state up into the app to save it in the URL
  const [scaleConfig, setScaleConfig] = useState(() =>
    fp.cloneDeep(defaultChartScaleConfig)
  );

  const enabledStats = trackedStatisticsFormFields.filter(
    ({ stat }) => config.dataConfig.trackedStatistics[stat]
  );

  const dataObservable = useMemo(
    () => simChartDatasetsObservable(simEvent$),
    [simEvent$]
  );

  return (
    <SortableMappedList keys={enabledStats} handle>
      {({ key, handleClassname }) => {
        const { stat, label: statName } = key;
        const availableScales = availableScalesByStat[stat];

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
                availableScales={availableScales}
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
