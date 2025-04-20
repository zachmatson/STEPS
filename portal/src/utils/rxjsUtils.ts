import { Observable, Subject } from "rxjs";

/**
 * Buffer values from the source observable until the returned observable gets a subscriber.
 *
 * The first subscriber will get all of the values that have been buffered; any subsequent subscribers are not
 * guaranteed to see values from before they have subscribed.
 *
 * Regardless of unsubscriptions, buffering will stop after the first subscriber subscribes and the buffer has been
 * drained.
 */
export function bufferUntilSubscribed<T>(source: Observable<T>): Observable<T> {
  // Underlying buffer which provides appropriate atomicity guarantees
  const buffer = new ObservableBuffer(source);
  // Subject which we will use to send values to observers
  const subject = new Subject<T>();
  // Track whether we have gotten any subscribers
  let hasBeenSubscribedTo = false;

  return new Observable<T>((observer) => {
    // Immediately subscribe to the subject
    // If this is the first subscriber, nothing will happen immediately, and we will begin to push to the subject below
    // For subsequent subscribers, this will either immediately start picking up values from the source, or it may
    // pick up some values being sent as the buffer drains.
    const subscription = subject.subscribe(observer);

    // Send values to first subscriber
    if (!hasBeenSubscribedTo) {
      hasBeenSubscribedTo = true;

      // Send all buffered values to the new subscriber
      // Atomically take chunks of values from the buffer and send them, in case observer.next has some side effect
      // which causes more values to be sent on the source, or yields to the event loop
      let bufferedValues;
      do {
        bufferedValues = buffer.takeAccumulatedValues();
        bufferedValues.forEach((value) => subject.next(value));
      } while (bufferedValues.length > 0);
      // Finally, once there are no buffered values left, we can atomically transfer the subscription to our subject
      buffer.bypassBuffer((value) => subject.next(value));
    }

    return () => {
      subscription.unsubscribe();
    };
  });
}

/**
 * Buffer an observable and use a "pull" model to get out buffered chunks
 */
class ObservableBuffer<T> {
  #buffer: T[] = [];
  #sourceObserver: (next: T) => void = () => {};

  constructor(source: Observable<T>) {
    // Set the buffer to "buffering" mode
    this.startBuffering();
    // Pass a closure capturing `this`. Then we can change #sourceObserver out without interacting with `source`.
    source.subscribe((next) => this.#sourceObserver(next));
  }

  startBuffering() {
    this.#sourceObserver = (next) => this.#buffer.push(next);
  }

  bypassBuffer(observer: (next: T) => void) {
    this.#sourceObserver = observer;
  }

  /**
   * Atomically take whichever values have accumulated since the last time this method was called
   */
  takeAccumulatedValues(): T[] {
    // Atomically replace buffer with empty list
    const existingValues = this.#buffer;
    this.#buffer = [];
    // Return buffered values
    return existingValues;
  }
}
