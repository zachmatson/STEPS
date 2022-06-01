import React from "react";

import fp from "lodash/fp";

import { PageLayout } from "./PageLayout";
import { Form, FormHandle } from "./form/Form";
import { ButtonFooter } from "./ButtonFooter";
import { NoResultsPlaceholder } from "./results/NoResultsPlaceholder";
import {
  decodeConfigFromURL,
  defaultPortalRunConfig,
  PortalRunConfig,
  portalRunConfigSchema,
  PortalRunConfigStringy,
  SimStatus,
} from "../config/config";
import { ResultsView } from "./results/ResultsView";
import { simChartDatasetsChannel } from "../charts/simChartDatasetsChannel";
import { SimWorkerLinkRxjs } from "../simulations/SimWorkerLinkRxjs";

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
  simWorkerLink = new SimWorkerLinkRxjs();
  chartDatasetsChannel = simChartDatasetsChannel(
    this.simWorkerLink.subscribables().results$
  );

  constructor(props: {}) {
    super(props);

    // TODO: Error checking and reporting
    // Load from URL
    let activeConfigStringy: PortalRunConfigStringy = fp.cloneDeep(
      defaultPortalRunConfig
    );
    const suppliedConfig = decodeConfigFromURL();
    if (suppliedConfig.success) {
      activeConfigStringy = suppliedConfig.data;
    }

    this.state = {
      status: "notStarted",
      activeConfigs: {
        numeric: portalRunConfigSchema.parse(activeConfigStringy),
        stringy: activeConfigStringy,
      },
      configIsDirty: false,
    };
    this.simWorkerLink
      .subscribables()
      .done$.subscribe(() => this.setState({ status: "finished" }));
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

    // TODO: Clean up the interface here maybe
    // TODO: Make the restarting/clearing a stream in the Link
    // Clear data from charts
    this.chartDatasetsChannel?.clear();

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
      // TODO: Status state based on Rxjs
      this.setState({ status: "paused" });
    }
  };

  resumeSim = () => {
    if (this.state.status == "paused") {
      this.simWorkerLink?.resume();
      // TODO: Status state based on Rxjs
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
              dataObservable={this.chartDatasetsChannel.datasets$}
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
