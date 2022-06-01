export type NarrowUnionByType<Union, Type> = Union extends { type: Type }
  ? Union
  : never;
