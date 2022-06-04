import { map, Observable, Subject } from "rxjs";

export const discardValues = () => map(() => {});

export const toSubscribedSubject =
  <T>(subjectFactory: () => Subject<T> = () => new Subject<T>()) =>
  (source$: Observable<T>): Observable<T> => {
    const subject = subjectFactory();
    source$.subscribe(subject);
    return subject.asObservable();
  };
