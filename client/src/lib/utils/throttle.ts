export function throttle<T extends Function>(f: T, limit: number = 300) {
  let lastFunc: NodeJS.Timeout;
  let lastRan: number;

  return function (...args: any[]) {
    // @ts-ignore
    const context: any = this;

    if (!lastRan) {
      f.apply(context, args);
      lastRan = Date.now();
    } else {
      clearTimeout(lastFunc);
      lastFunc = setTimeout(function () {
        if (Date.now() - lastRan >= limit) {
          f.apply(context, args);
          lastRan = Date.now();
        }
      }, limit - (Date.now() - lastRan));
    }
  };
}
