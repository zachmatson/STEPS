export type FilterUnionByType<UnionInstance, Type> = UnionInstance extends {
  type: Type;
}
  ? UnionInstance
  : never;
