import React from "react";

import fp from "lodash/fp";
import { decodeConfigFromURL } from "../config/configEncoding";

import { PageLayout } from "./PageLayout";
import { Form, FormHandle } from "./form/Form";
import { ButtonFooter } from "./ButtonFooter";
import { NoResultsPlaceholder } from "./results/NoResultsPlaceholder";
import {
  defaultPortalRunConfig,
  PortalRunConfig,
  portalRunConfigSchema,
  PortalRunConfigStringy,
  SimStatus,
} from "../config/PortalRunConfig";
import { ResultsView } from "./results/ResultsView";
import { simChartDatasetsObservable } from "../charts/simChartDatasetsObservable";
import { SimLink } from "../simulations/SimLink";

type AppState = {
  status: SimStatus;
  activeConfigs: {
    numeric: PortalRunConfig;
    stringy: PortalRunConfigStringy;
  };
  configIsDirty: boolean;
  csvDownloadUrl: string | null;
};

export class App extends React.Component<{}, AppState> {
  formRef = React.createRef<FormHandle>();
  simLink = new SimLink();
  chartDatasets$ = simChartDatasetsObservable(this.simLink.observables());

  constructor(props: {}) {
    super(props);

    // Load from URL
    const suppliedConfig = decodeConfigFromURL();
    const activeConfigStringy = suppliedConfig.success
      ? suppliedConfig.data
      : fp.cloneDeep(defaultPortalRunConfig);

    this.simLink.observables().outcome$.subscribe((outcome) =>
      this.setState({
        status: "finished",
        csvDownloadUrl: outcome.csvDownloadUrl ?? null,
      })
    );

    this.state = {
      status: "notStarted",
      activeConfigs: {
        numeric: portalRunConfigSchema.parse(activeConfigStringy),
        stringy: activeConfigStringy,
      },
      configIsDirty: false,
      csvDownloadUrl: null,
    };
  }

  setConfigIsDirty = (isDirty: boolean) => {
    this.setState({
      configIsDirty: isDirty,
    });
  };

  triggerFormSubmit = () => {
    this.formRef.current?.submit();
  };

  handleFormSubmit = (
    config: PortalRunConfig,
    configStringy: PortalRunConfigStringy
  ) => {
    this.simLink.startOrRestart(config);

    this.setState({
      status: "running",
      activeConfigs: {
        numeric: config,
        stringy: configStringy,
      },
      configIsDirty: false,
      csvDownloadUrl: null,
    });
  };

  pauseSim = () => {
    if (this.state.status == "running") {
      this.simLink?.pause();
      this.setState({ status: "paused" });
    }
  };

  resumeSim = () => {
    if (this.state.status == "paused") {
      this.simLink?.resume();
      this.setState({ status: "running" });
    }
  };

  render() {
    return (
      <PageLayout
        form={
          <Form
            ref={this.formRef}
            defaultValues={this.state.activeConfigs.stringy}
            onSubmit={this.handleFormSubmit}
            onDirtinessChange={this.setConfigIsDirty}
          />
        }
        results={
          this.state.status == "notStarted" ? (
            <NoResultsPlaceholder />
          ) : (
            <ResultsView
              config={this.state.activeConfigs.numeric}
              dataObservable={this.chartDatasets$}
            />
          )
        }
        footer={
          <ButtonFooter
            status={this.state.status}
            configs={this.state.activeConfigs}
            configIsDirty={this.state.configIsDirty}
            csvDownloadUrl={this.state.csvDownloadUrl ?? undefined}
            startOrRestartSim={this.triggerFormSubmit}
            pauseSim={this.pauseSim}
            resumeSim={this.resumeSim}
          />
        }
      />
    );
  }
}
