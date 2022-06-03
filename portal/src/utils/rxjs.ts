import {
  connectable,
  Observable,
  ObservableInput,
  Subject,
  SubjectLike,
} from "rxjs";

export const makeHotImmediate =
  <T>(connector: () => SubjectLike<T> = () => new Subject<T>()) =>
  (source: ObservableInput<T>): Observable<T> => {
    const sourceShared$ = connectable(source, {
      connector,
      resetOnDisconnect: false,
    });
    sourceShared$.connect();
    return sourceShared$;
  };
