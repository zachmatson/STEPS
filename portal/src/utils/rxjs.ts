import {
  connectable,
  map,
  Observable,
  ObservableInput,
  SubjectLike,
} from "rxjs";

export const ignoreValue = () => map(() => {});

export const connectWithSubject =
  <T>(connector: () => SubjectLike<T>) =>
  (source: ObservableInput<T>): Observable<T> => {
    const sourceShared$ = connectable(source, {
      connector,
      resetOnDisconnect: false,
    });
    sourceShared$.connect();
    return sourceShared$;
  };
