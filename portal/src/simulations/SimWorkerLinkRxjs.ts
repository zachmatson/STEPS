import {
  filter,
  fromEvent,
  map,
  Observable,
  share,
  Subject,
  switchMap,
  take,
  takeUntil,
} from "rxjs";

import {
  OutboundSimWorkerMessage,
  SimResultsFragment,
  SimWorkerHandle,
} from "./workerInterface";
import { PortalRunConfig } from "../config/config";
import { NarrowUnionByType } from "../utils/NarrowUnionByType";
import { ignoreValue } from "../utils/rxjs";

const filterMessageType =
  <Type extends OutboundSimWorkerMessage["type"]>(type: Type) =>
  (source$: Observable<OutboundSimWorkerMessage>) =>
    source$.pipe(
      filter((msg: OutboundSimWorkerMessage) => msg.type == type),
      map((msg) => msg as NarrowUnionByType<OutboundSimWorkerMessage, Type>)
    );

export interface SimSubscribables {
  results$: Observable<SimResultsFragment[]>;
  done$: Observable<void>;
  paused$: Observable<void>;
  resumed$: Observable<void>;
}

export class SimWorkerLinkRxjs {
  #worker$: Subject<SimWorkerHandle>;
  #currentWorkerHandle: SimWorkerHandle | null = null;
  #incomingMessage$: Observable<OutboundSimWorkerMessage>;
  #ready$: Observable<void>;
  #results$: Observable<SimResultsFragment[]>;
  #done$: Observable<void>;
  #paused$: Subject<void>;
  #resumed$: Subject<void>;

  constructor() {
    this.#worker$ = new Subject<SimWorkerHandle>();
    this.#incomingMessage$ = this.#worker$.pipe(
      switchMap((worker) =>
        fromEvent<MessageEvent>(worker, "message").pipe(
          map((msg) => msg.data as OutboundSimWorkerMessage)
        )
      ),
      share()
    );
    this.#ready$ = this.#incomingMessage$.pipe(
      filterMessageType("ready"),
      ignoreValue(),
      share()
    );
    this.#results$ = this.#incomingMessage$.pipe(
      filterMessageType("results"),
      map((msg) => msg.results),
      share()
    );
    this.#done$ = this.#incomingMessage$.pipe(
      filterMessageType("done"),
      ignoreValue(),
      share()
    );
    // TODO: Declarative status observable
    this.#paused$ = new Subject<void>();
    this.#paused$.subscribe(() =>
      this.#currentWorkerHandle?.postMessage({ type: "pause" })
    );
    this.#resumed$ = new Subject<void>();
    this.#resumed$.subscribe(() =>
      this.#currentWorkerHandle?.postMessage({ type: "resume" })
    );
  }

  startOrRestart(config: PortalRunConfig) {
    this.#currentWorkerHandle?.terminate();
    const worker = new Worker(
      new URL("./worker.ts", import.meta.url)
    ) as unknown as SimWorkerHandle;
    this.#currentWorkerHandle = worker;
    this.#worker$.next(worker);

    this.#ready$.pipe(takeUntil(this.#worker$), take(1)).subscribe(() =>
      worker.postMessage({
        type: "start",
        config: config,
      })
    );
  }

  pause() {
    this.#paused$.next();
  }

  resume() {
    this.#resumed$.next();
  }

  subscribables(): SimSubscribables {
    return {
      results$: this.#results$,
      done$: this.#done$,
      paused$: this.#paused$,
      resumed$: this.#resumed$,
    };
  }
}
