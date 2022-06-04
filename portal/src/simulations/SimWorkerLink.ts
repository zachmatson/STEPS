import {
  fromEvent,
  Observable,
  Subject,
  map,
  switchMap,
  share,
  withLatestFrom,
  scan,
  filter,
} from "rxjs";

import { discardValues } from "../utils/rxjs";
import { FilterUnionByType } from "../utils/typescript";
import {
  InboundSimWorkerMessage,
  OutboundSimWorkerMessage,
  SimResultsFragments,
  SimWorkerHandle,
} from "./workerInterface";
import { PortalRunConfig } from "../config/config";

export interface SimObservables {
  start$: Observable<PortalRunConfig>;
  results$: Observable<SimResultsFragments>;
  done$: Observable<void>;
}

export class SimWorkerLink {
  #start$: Subject<PortalRunConfig>;
  #messagesToWorker$: Subject<InboundSimWorkerMessage>;
  #publicObservables: SimObservables;

  constructor() {
    this.#start$ = new Subject();
    const worker$: Observable<SimWorkerHandle> = this.#start$.pipe(
      scan<unknown, SimWorkerHandle, null>((handle) => {
        handle?.terminate();
        return new Worker(
          new URL("./worker.ts", import.meta.url)
        ) as unknown as SimWorkerHandle;
      }, null),
      share()
    );

    const messagesFromWorker$: Observable<OutboundSimWorkerMessage> =
      worker$.pipe(
        switchMap((worker) =>
          fromEvent<MessageEvent<OutboundSimWorkerMessage>>(worker, "message")
        ),
        map((msg) => msg.data),
        share()
      );

    this.#messagesToWorker$ = new Subject();
    this.#messagesToWorker$
      .pipe(withLatestFrom(worker$))
      .subscribe(([message, worker]) => {
        worker.postMessage(message);
      });

    messagesFromWorker$
      .pipe(
        filterMessageByType("ready"),
        withLatestFrom(this.#start$),
        map<[unknown, PortalRunConfig], InboundSimWorkerMessage>(
          ([_, config]) => ({
            type: "start",
            config,
          })
        )
      )
      .subscribe(this.#messagesToWorker$);

    this.#publicObservables = {
      start$: this.#start$.asObservable(),
      results$: messagesFromWorker$.pipe(
        filterMessageByType("results"),
        map(({ results }) => results),
        share()
      ),
      done$: messagesFromWorker$.pipe(
        filterMessageByType("done"),
        discardValues(),
        share()
      ),
    };
  }

  startOrRestart(config: PortalRunConfig) {
    this.#start$.next(config);
  }

  pause() {
    this.#messagesToWorker$.next({ type: "pause" });
  }

  resume() {
    this.#messagesToWorker$.next({ type: "resume" });
  }

  observables() {
    return this.#publicObservables;
  }
}

const filterMessageByType =
  <Type extends OutboundSimWorkerMessage["type"]>(t: Type) =>
  (source$: Observable<OutboundSimWorkerMessage>) =>
    source$.pipe(filter((msg) => msg.type == t)) as Observable<
      FilterUnionByType<OutboundSimWorkerMessage, Type>
    >;
