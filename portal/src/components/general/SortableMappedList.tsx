import React, { useEffect, useMemo, useState } from "react";

import { ReactSortable, ItemInterface } from "react-sortablejs";
import { v4 as uuidv4 } from "uuid";
import fp from "lodash/fp";

export type SortableMappedListRenderInput = {
  key: string;
  handleClass?: string;
};

export type SortableMappedListProps = {
  keys: string[];
  handle?: boolean | string;
  children: (key: SortableMappedListRenderInput) => React.ReactNode;
};

export const SortabbleMappedList = ({
  keys,
  handle,
  children: render,
}: SortableMappedListProps) => {
  const handleClass = useMemo<string | undefined>(
    () => (fp.isString(handle) || handle === undefined ? handle : uuidv4()),
    [handle]
  );

  console.log(handleClass);

  const [list, setList] = useState<ItemInterface[]>([]);

  // Handle changes in keys
  useEffect(() => {
    setList((oldList) => updateListForKeys(oldList, keys));
  }, [keys]);

  return (
    <ReactSortable list={list} setList={setList} handle={`.${handleClass}`}>
      {list.map((item) => render({ key: item.id as string, handleClass }))}
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
