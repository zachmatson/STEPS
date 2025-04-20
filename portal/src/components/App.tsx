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
import { SimLink } from "../simulations/SimLink";
import { bufferUntilSubscribed } from "../utils/rxjsUtils";

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
  // Create an observable that will be buffered until the ResultsView starts to read from it.
  // This is out of extreme caution, the results view should be able to subscribe before simulations can be started.
  simLinkEventsForResultsView$ = bufferUntilSubscribed(
    this.simLink.eventsObservable()
  );

  constructor(props: {}) {
    super(props);

    // Load from URL
    const suppliedConfig = decodeConfigFromURL();
    const activeConfigStringy = suppliedConfig.success
      ? suppliedConfig.data
      : fp.cloneDeep(defaultPortalRunConfig);

    this.simLink.eventsObservable().subscribe((event) => {
      if (event.type == "done") {
        this.setState({
          status: "finished",
          csvDownloadUrl: event.outcome.csvDownloadUrl ?? null,
        });
      }
    });

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
              simEvent$={this.simLinkEventsForResultsView$}
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
