import type { DocumentKind, TaskPriority, TaskStatus, TaskType } from "./api-types";

export type Route =
  | { name: "overview" }
  | { name: "documents" }
  | { name: "specs" }
  | { name: "designs" }
  | { name: "adrs" }
  | { name: "tasks" }
  | { name: "task-detail"; id: number }
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

export type TaskFilterState = {
  status: "all" | TaskStatus;
  type: "all" | TaskType;
  priority: "all" | TaskPriority;
  tag: string;
};

export const initialTaskFilters: TaskFilterState = {
  status: "all",
  type: "all",
  priority: "all",
  tag: "",
};
