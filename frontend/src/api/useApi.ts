import { useEffect, useState } from "react";

export function useApi<T>(loader: () => Promise<T>, dependencies: unknown[] = []) {
  const [data, setData] = useState<T>();
  const [error, setError] = useState<Error>();
  useEffect(() => {
    let active = true;
    setData(undefined);
    setError(undefined);
    loader().then(
      (value) => active && setData(value),
      (reason: unknown) =>
        active && setError(reason instanceof Error ? reason : new Error(String(reason))),
    );
    return () => {
      active = false;
    };
    // Loader functions are intentionally represented by the caller's stable primitive dependencies.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, dependencies);
  return { data, error };
}
