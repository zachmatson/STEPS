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
import { SimWorkerLink } from "../simulations/SimWorkerLink";

type AppState = {
  status: SimStatus;
  activeConfigs: {
    numeric: PortalRunConfig;
    stringy: PortalRunConfigStringy;
  };
  configIsDirty: boolean;
};

export class App extends React.Component<{}, AppState> {
  formRef = React.createRef<FormHandle>();
  simWorkerLink = new SimWorkerLink();
  chartDatasets$ = simChartDatasetsObservable(this.simWorkerLink.observables());

  constructor(props: {}) {
    super(props);

    // Load from URL
    let activeConfigStringy: PortalRunConfigStringy = fp.cloneDeep(
      defaultPortalRunConfig
    );
    const suppliedConfig = decodeConfigFromURL();
    if (suppliedConfig.success) {
      activeConfigStringy = suppliedConfig.data;
    }

    this.simWorkerLink
      .observables()
      .done$.subscribe(() => this.setState({ status: "finished" }));

    this.state = {
      status: "notStarted",
      activeConfigs: {
        numeric: portalRunConfigSchema.parse(activeConfigStringy),
        stringy: activeConfigStringy,
      },
      configIsDirty: false,
    };
  }

  triggerFormSubmit = () => {
    this.formRef.current?.submit();
  };

  setConfigIsDirty = (isDirty: boolean) => {
    this.setState({
      configIsDirty: isDirty,
    });
  };

  startOrRestartSim = () => {
    const configStringy = fp.cloneDeep(this.formRef.current?.getValues());
    if (!configStringy) return;
    const config = portalRunConfigSchema.parse(configStringy);

    this.simWorkerLink.startOrRestart(config);

    this.setState(
      {
        status: "running",
        activeConfigs: {
          numeric: config,
          stringy: configStringy,
        },
        configIsDirty: false,
      },
      () => {
        this.formRef.current?.reset(configStringy, {
          keepValues: true,
        });
      }
    );
  };

  pauseSim = () => {
    if (this.state.status == "running") {
      this.simWorkerLink?.pause();
      this.setState({ status: "paused" });
    }
  };

  resumeSim = () => {
    if (this.state.status == "paused") {
      this.simWorkerLink?.resume();
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
            onSubmit={this.startOrRestartSim}
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
            startOrRestartSim={this.triggerFormSubmit}
            pauseSim={this.pauseSim}
            resumeSim={this.resumeSim}
          />
        }
      />
    );
  }
}
