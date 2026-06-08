import { useEffect, useState } from "react";

import { StatusPill } from "./components/chrome";
import type {
  AdrSummary,
  DesignSummary,
  DocumentSummary,
  HealthResponse,
  SpecSummary,
  TaskGroupsResponse,
  ValidationResponse,
} from "./lib/api-types";
import { loadJson } from "./lib/api";
import type { LoadState, Route } from "./lib/app-types";
import { isActiveRoute, navItems, parseRoute } from "./lib/routing";
import { DesignsScreen } from "./screens/DesignsScreen";
import { DocumentDetailScreen } from "./screens/DocumentDetailScreen";
import { DocumentsScreen } from "./screens/DocumentsScreen";
import { Overview } from "./screens/Overview";
import { SpecsScreen } from "./screens/SpecsScreen";
import { AdrsScreen } from "./screens/AdrsScreen";
import { TaskDetailScreen } from "./screens/TaskDetailScreen";
import { TasksScreen } from "./screens/TasksScreen";

export function App() {
  const [route, setRoute] = useState<Route>(() => parseRoute(window.location.pathname));
  const [documents, setDocuments] = useState<LoadState<DocumentSummary[]>>({ status: "loading" });
  const [specs, setSpecs] = useState<LoadState<SpecSummary[]>>({ status: "loading" });
  const [designs, setDesigns] = useState<LoadState<DesignSummary[]>>({ status: "loading" });
  const [adrs, setAdrs] = useState<LoadState<AdrSummary[]>>({ status: "loading" });
  const [tasks, setTasks] = useState<LoadState<TaskGroupsResponse>>({ status: "loading" });
  const [validation, setValidation] = useState<LoadState<ValidationResponse>>({ status: "loading" });
  const [health, setHealth] = useState<LoadState<HealthResponse>>({ status: "loading" });

  useEffect(() => {
    const onPopState = () => setRoute(parseRoute(window.location.pathname));
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, []);

  useEffect(() => {
    void loadJson<HealthResponse>("/api/health").then(setHealth);
    void loadJson<DocumentSummary[]>("/api/documents").then(setDocuments);
    void loadJson<SpecSummary[]>("/api/specs").then(setSpecs);
    void loadJson<DesignSummary[]>("/api/designs").then(setDesigns);
    void loadJson<AdrSummary[]>("/api/adr").then(setAdrs);
    void loadJson<TaskGroupsResponse>("/api/tasks").then(setTasks);
    void loadJson<ValidationResponse>("/api/validation").then(setValidation);
  }, []);

  function navigate(next: Route, href: string) {
    window.history.pushState(null, "", href);
    setRoute(next);
  }

  const documentList = documents.status === "ready" ? documents.data : [];
  const validationIssues = validation.status === "ready" ? validation.data.issues : [];

  return (
    <div className="min-h-screen bg-surface text-ink">
      <header className="border-b border-line bg-surface-raised/90 backdrop-blur">
        <div className="mx-auto flex max-w-7xl flex-col gap-5 px-4 py-6 sm:px-6 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="text-xs font-semibold uppercase text-action" translate="no">vibe-doc</p>
            <h1 className="font-display text-4xl font-normal tracking-normal sm:text-5xl">Repository Journal</h1>
          </div>
          <StatusPill health={health} validation={validation} />
        </div>
      </header>

      <div className="mx-auto grid max-w-7xl gap-7 px-4 py-7 sm:px-6 lg:grid-cols-[220px_1fr]">
        <aside className="h-fit lg:sticky lg:top-6 lg:border-r lg:border-line lg:pr-4">
          <nav aria-label="Primary screens" className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:block lg:space-y-2">
            {navItems.map((item) => (
              <button
                className={`flex min-h-10 items-center justify-between rounded-full border px-4 py-2 text-left text-sm font-semibold transition lg:w-full ${
                  isActiveRoute(route, item.route)
                    ? "border-action-border bg-action-soft text-action"
                    : "border-line bg-surface-raised/80 text-ink-muted hover:border-action-border hover:bg-surface-muted hover:text-ink"
                }`}
                key={item.href}
                onClick={() => navigate(item.route, item.href)}
                type="button"
              >
                <span>{item.label}</span>
              </button>
            ))}
          </nav>
        </aside>

        <main className="min-w-0">
          {route.name === "overview" ? (
            <Overview
              documents={documents}
              health={health}
              navigate={navigate}
              validation={validation}
            />
          ) : null}
          {route.name === "documents" ? (
            <DocumentsScreen
              documents={documents}
              navigate={navigate}
              validationIssues={validationIssues}
            />
          ) : null}
          {route.name === "specs" ? (
            <SpecsScreen
              navigate={navigate}
              specs={specs}
              validationIssues={validationIssues}
            />
          ) : null}
          {route.name === "designs" ? (
            <DesignsScreen
              designs={designs}
              navigate={navigate}
              validationIssues={validationIssues}
            />
          ) : null}
          {route.name === "adrs" ? (
            <AdrsScreen
              adrs={adrs}
              navigate={navigate}
              validationIssues={validationIssues}
            />
          ) : null}
          {route.name === "tasks" ? (
            <TasksScreen
              navigate={navigate}
              tasks={tasks}
              validationIssues={validationIssues}
            />
          ) : null}
          {route.name === "task-detail" ? (
            <TaskDetailScreen
              documents={documentList}
              id={route.id}
              navigate={navigate}
              validationIssues={validationIssues}
            />
          ) : null}
          {route.name === "detail" ? (
            <DocumentDetailScreen
              documents={documentList}
              id={route.id}
              navigate={navigate}
              validationIssues={validationIssues}
            />
          ) : null}
        </main>
      </div>
    </div>
  );
}
