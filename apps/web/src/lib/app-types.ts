import type { DocumentKind } from "./api-types";

export type Route =
  | { name: "overview" }
  | { name: "documents" }
  | { name: "specs" }
  | { name: "designs" }
  | { name: "detail"; id: number };

export type Navigate = (route: Route, href: string) => void;

export type LoadState<T> =
  | { status: "loading" }
  | { status: "error"; message: string }
  | { status: "ready"; data: T };

export type FilterState = {
  kind: "all" | DocumentKind;
  tag: string;
  title: string;
  id: string;
};

export const initialFilters: FilterState = {
  kind: "all",
  tag: "",
  title: "",
  id: "",
};
