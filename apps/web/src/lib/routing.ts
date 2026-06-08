import type { Route } from "./app-types";

export const navItems: Array<{ label: string; href: string; route: Route }> = [
  { label: "Overview", href: "/", route: { name: "overview" } },
  { label: "Documents", href: "/documents", route: { name: "documents" } },
  { label: "Specs", href: "/specs", route: { name: "specs" } },
  { label: "Designs", href: "/designs", route: { name: "designs" } },
  { label: "ADRs", href: "/adr", route: { name: "adrs" } },
  { label: "Tasks", href: "/tasks", route: { name: "tasks" } },
];

export function parseRoute(pathname: string): Route {
  const detail = pathname.match(/^\/documents\/(\d+)$/);
  if (detail?.[1]) {
    return { name: "detail", id: Number(detail[1]) };
  }
  const taskDetail = pathname.match(/^\/tasks\/(\d+)$/);
  if (taskDetail?.[1]) {
    return { name: "task-detail", id: Number(taskDetail[1]) };
  }
  if (pathname === "/documents") {
    return { name: "documents" };
  }
  if (pathname === "/specs") {
    return { name: "specs" };
  }
  if (pathname === "/designs") {
    return { name: "designs" };
  }
  if (pathname === "/adr") {
    return { name: "adrs" };
  }
  if (pathname === "/tasks") {
    return { name: "tasks" };
  }
  return { name: "overview" };
}

export function isActiveRoute(current: Route, target: Route) {
  return current.name === target.name
    || (current.name === "detail" && target.name === "documents")
    || (current.name === "task-detail" && target.name === "tasks");
}
