type What<E> = {
  [K in keyof E]: {
    t: K;
    c: E[K];
  };
};

export type Enum<E> = What<E>[keyof What<E>];
