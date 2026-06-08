import { useEffect, useState } from "react";

import { StatusPill } from "./components/chrome";
import type {
  DesignSummary,
  DocumentSummary,
  HealthResponse,
  SpecSummary,
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

export function App() {
  const [route, setRoute] = useState<Route>(() => parseRoute(window.location.pathname));
  const [documents, setDocuments] = useState<LoadState<DocumentSummary[]>>({ status: "loading" });
  const [specs, setSpecs] = useState<LoadState<SpecSummary[]>>({ status: "loading" });
  const [designs, setDesigns] = useState<LoadState<DesignSummary[]>>({ status: "loading" });
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
      <header className="border-b border-slate-200 bg-surface-raised">
        <div className="mx-auto flex max-w-7xl flex-col gap-4 px-4 py-4 sm:px-6 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <p className="text-sm font-medium text-action">vibe-doc</p>
            <h1 className="text-2xl font-semibold tracking-normal">Repository docs</h1>
          </div>
          <StatusPill health={health} validation={validation} />
        </div>
      </header>

      <div className="mx-auto grid max-w-7xl gap-6 px-4 py-5 sm:px-6 lg:grid-cols-[220px_1fr]">
        <aside className="h-fit lg:border-r lg:border-slate-200 lg:pr-4">
          <nav aria-label="Primary screens" className="grid grid-cols-2 gap-2 sm:grid-cols-4 lg:block lg:space-y-1">
            {navItems.map((item) => (
              <button
                className={`flex min-h-10 items-center justify-between rounded border px-3 py-2 text-left text-sm font-medium lg:w-full ${
                  isActiveRoute(route, item.route)
                    ? "border-action bg-action-soft text-action"
                    : "border-slate-200 bg-surface-raised text-ink-muted hover:bg-surface-muted hover:text-ink"
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
