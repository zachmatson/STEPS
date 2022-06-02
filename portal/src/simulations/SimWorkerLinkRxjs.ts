import {
  filter,
  fromEvent,
  map,
  Observable,
  share,
  Subject,
  switchMap,
  withLatestFrom,
} from "rxjs";

import {
  InboundSimWorkerMessage,
  OutboundSimWorkerMessage,
  OutboundSimWorkerMessageResults,
  SimResultsFragment,
  SimWorkerHandle,
} from "./workerInterface";
import { PortalRunConfig } from "../config/config";
import { NarrowUnionByType } from "../utils/NarrowUnionByType";
import { ignoreValue, makeHotImmediate } from "../utils/rxjs";

const filterMessageType =
  <Type extends OutboundSimWorkerMessage["type"]>(type: Type) =>
  (source$: Observable<OutboundSimWorkerMessage>) =>
    source$.pipe(
      filter((msg: OutboundSimWorkerMessage) => msg.type == type),
      map((msg) => msg as NarrowUnionByType<OutboundSimWorkerMessage, Type>)
    );

export interface SimObservables2 {
  start$: Observable<void>;
  results$: Observable<SimResultsFragment[]>;
  done$: Observable<void>;
}

export class SimWorkerLinkRxjs {
  #startWithConfig$: Subject<PortalRunConfig> = new Subject();
  #workerHandle$ = this.#startWithConfig$.pipe(
    map(
      () =>
        new Worker(
          new URL("./worker.ts", import.meta.url)
        ) as unknown as SimWorkerHandle
    ),
    makeHotImmediate()
  );
  #incomingMessage$ = this.#workerHandle$.pipe(
    switchMap((handle) =>
      fromEvent<MessageEvent>(handle, "message").pipe(
        map((msg) => msg.data as OutboundSimWorkerMessage)
      )
    ),
    share()
  );
  #outgoingMessage$: Subject<InboundSimWorkerMessage> = new Subject();
  #ready$ = this.#incomingMessage$.pipe(
    filterMessageType("ready"),
    ignoreValue(),
    share()
  );
  #result$ = this.#incomingMessage$.pipe(
    filterMessageType("results"),
    map<OutboundSimWorkerMessageResults, SimResultsFragment[]>(
      (data) => data.results
    ),
    share()
  );
  #done$ = this.#incomingMessage$.pipe(
    filterMessageType("done"),
    ignoreValue(),
    share()
  );
  #observablesExternal$: SimObservables2 = {
    start$: this.#startWithConfig$.pipe(ignoreValue()),
    results$: this.#result$,
    done$: this.#done$,
  };

  constructor() {
    // Send outgoing messages
    this.#outgoingMessage$
      .pipe(withLatestFrom(this.#workerHandle$))
      .subscribe(([message, workerHandle]) => {
        workerHandle.postMessage(message);
      });

    // Start worker with config when it is ready
    this.#ready$
      .pipe(withLatestFrom(this.#startWithConfig$))
      .subscribe(([_, config]) => {
        this.#outgoingMessage$.next({ type: "start", config });
      });
  }

  startOrRestart(config: PortalRunConfig) {
    this.#startWithConfig$.next(config);
  }

  pause() {
    this.#outgoingMessage$.next({ type: "pause" });
  }

  resume() {
    this.#outgoingMessage$.next({ type: "resume" });
  }

  observables(): SimObservables2 {
    return this.#observablesExternal$;
  }
}
