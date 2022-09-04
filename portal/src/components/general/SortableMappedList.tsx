import React, { useEffect, useState } from "react";

import { ReactSortable, ItemInterface } from "react-sortablejs";

export type SortableMappedListProps = {
  keys: string[];
  renderKey: (key: string) => React.ReactNode;
};

export const SortabbleMappedList = ({
  keys,
  renderKey,
}: SortableMappedListProps) => {
  const [list, setList] = useState<ItemInterface[]>([]);

  // Handle changes in keys
  useEffect(() => {
    setList((oldList) => updateListForKeys(oldList, keys));
  }, [keys]);

  return (
    <ReactSortable list={list} setList={setList}>
      {list.map((item) => renderKey(item.id as string))}
    </ReactSortable>
  );
};

const keysToItemInterface = (key: string): ItemInterface => ({ id: key });
const itemInterfaceToKeys = (item: ItemInterface) => item.id;
const updateListForKeys = (oldList: ItemInterface[], keys: string[]) => {
  const oldListSet = new Set(oldList.map(itemInterfaceToKeys));
  const keysSet = new Set(keys);

  return [
    ...oldList.filter((item) => keysSet.has(item.id as string)),
    ...keys.filter((key) => !oldListSet.has(key)).map(keysToItemInterface),
  ];
};
