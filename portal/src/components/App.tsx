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
import { SimWorkerLink } from "../simulations/SimWorkerLink";
import { ResultsView } from "./results/ResultsView";
import { SimChartDatasetsBehaviorSubject } from "../charts/SimChartDatasetsBehaviorSubject";

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
  simWorkerLink: SimWorkerLink | undefined;
  chartDataSubject = new SimChartDatasetsBehaviorSubject([]);

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
    // Use getValue instead because we want the stringy data here
    // This submit handler gets a data argument too but it is the wrong type,
    // and React-Hook-Form's typing doesn't understand the concept of input vs
    // output types in zod
    const dataStringy = fp.cloneDeep(this.formRef.current?.getValues());
    if (!dataStringy) return;
    const data = portalRunConfigSchema.parse(dataStringy);

    // Need to clear data from all charts by pushing empty dataset
    this.chartDataSubject.resetData();

    // TODO: Clean up the interface here maybe
    this.simWorkerLink?.terminate();
    this.simWorkerLink = new SimWorkerLink({
      onResults: (newResults) => {
        this.chartDataSubject.nextFromFragments(newResults);
      },
      onFinish: () => this.setState({ status: "finished" }),
    });
    this.simWorkerLink.start(data);

    this.setState(
      {
        status: "running",
        activeConfigs: {
          numeric: data,
          stringy: dataStringy,
        },
        configIsDirty: false,
      },
      () => {
        this.formRef.current?.reset(dataStringy, {
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
              dataObservable={this.chartDataSubject}
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
