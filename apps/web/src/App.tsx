import type {
  ApiRoute,
  DocumentKind,
  OverviewResponse,
  TaskStatus,
  ValidationSeverity,
} from "./lib/api-types";

const overview: OverviewResponse = {
  document_count: 25,
  active_task_count: 6,
  done_task_count: 12,
  adr_count: 0,
  validation: {
    status: "unknown",
    error_count: 0,
    warning_count: 0,
  },
  recently_updated: [
    {
      id: 27,
      title: "Scaffold Web UI app and shared API types",
      kind: "task",
      path: "docs/tasks/active/27-scaffold-web-ui-app-and-shared-api-types.md",
    },
    {
      id: 34,
      title: "vibe-doc Web Server and UI Design",
      kind: "design",
      path: "docs/designs/34-vibe-doc-web-server-design.md",
    },
  ],
};

const screens: Array<{ label: string; route: string; kind?: DocumentKind }> = [
  { label: "Overview", route: "/" },
  { label: "Documents", route: "/documents" },
  { label: "Specs", route: "/specs", kind: "spec" },
  { label: "Designs", route: "/designs", kind: "design" },
  { label: "ADRs", route: "/adr", kind: "adr" },
  { label: "Tasks", route: "/tasks", kind: "task" },
  { label: "Validation", route: "/validation" },
];

const apiRoutes: ApiRoute[] = [
  { method: "GET", path: "/api/health", description: "Server readiness" },
  { method: "GET", path: "/api/documents", description: "Numbered documents" },
  { method: "GET", path: "/api/documents/:id", description: "Document detail" },
  { method: "GET", path: "/api/tasks", description: "Task groups" },
  { method: "GET", path: "/api/validation", description: "Validation report" },
  { method: "GET", path: "/api/context/task/:id", description: "Task context" },
];

const taskStatuses: TaskStatus[] = ["planned", "doing", "blocked", "done", "dropped"];
const severities: ValidationSeverity[] = ["error", "warning", "info"];

export function App() {
  return (
    <div className="min-h-screen bg-surface text-ink">
      <header className="border-b border-slate-200 bg-surface-raised">
        <div className="mx-auto flex max-w-7xl items-center justify-between gap-6 px-6 py-4">
          <div>
            <p className="text-sm font-medium text-action">vibe-doc</p>
            <h1 className="text-2xl font-semibold tracking-normal">Repository docs</h1>
          </div>
          <div className="rounded border border-slate-200 px-3 py-2 text-sm text-ink-muted">
            API status: <span className="font-medium text-attention">pending</span>
          </div>
        </div>
      </header>

      <div className="mx-auto grid max-w-7xl gap-6 px-6 py-6 lg:grid-cols-[220px_1fr]">
        <aside className="h-fit border-r border-slate-200 pr-4">
          <nav aria-label="Primary screens" className="space-y-1">
            {screens.map((screen) => (
              <a
                className="flex items-center justify-between rounded px-3 py-2 text-sm font-medium text-ink-muted hover:bg-surface-muted hover:text-ink"
                href={screen.route}
                key={screen.route}
              >
                <span>{screen.label}</span>
                {screen.kind ? (
                  <span className="text-xs font-normal text-ink-soft" translate="no">
                    {screen.kind}
                  </span>
                ) : null}
              </a>
            ))}
          </nav>
        </aside>

        <main className="space-y-6">
          <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4" aria-label="Repository summary">
            <SummaryMetric label="Documents" value={overview.document_count} />
            <SummaryMetric label="Active tasks" value={overview.active_task_count} />
            <SummaryMetric label="Done tasks" value={overview.done_task_count} />
            <SummaryMetric label="ADRs" value={overview.adr_count} />
          </section>

          <section className="grid gap-6 xl:grid-cols-[1fr_360px]">
            <div className="space-y-4">
              <div className="flex items-center justify-between gap-4">
                <h2 className="text-lg font-semibold">Recent documents</h2>
                <span className="rounded bg-attention-soft px-2 py-1 text-xs font-medium text-attention">
                  Read-only
                </span>
              </div>
              <div className="overflow-hidden rounded border border-slate-200 bg-surface-raised">
                <table className="w-full table-fixed text-left text-sm">
                  <thead className="bg-surface-muted text-xs uppercase text-ink-soft">
                    <tr>
                      <th className="w-20 px-4 py-3">ID</th>
                      <th className="px-4 py-3">Title</th>
                      <th className="w-28 px-4 py-3">Kind</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-200">
                    {overview.recently_updated.map((document) => (
                      <tr key={document.id}>
                        <td className="px-4 py-3 font-medium" translate="no">
                          {document.id}
                        </td>
                        <td className="px-4 py-3">
                          <div className="truncate font-medium">{document.title}</div>
                          <div className="truncate text-xs text-ink-soft" translate="no">
                            {document.path}
                          </div>
                        </td>
                        <td className="px-4 py-3 text-ink-muted" translate="no">
                          {document.kind}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>

            <div className="space-y-4">
              <h2 className="text-lg font-semibold">API contract</h2>
              <div className="space-y-2">
                {apiRoutes.map((route) => (
                  <div className="rounded border border-slate-200 bg-surface-raised p-3" key={route.path}>
                    <div className="flex items-center gap-2 text-sm">
                      <span className="rounded bg-action-soft px-2 py-1 text-xs font-semibold text-action" translate="no">
                        {route.method}
                      </span>
                      <span className="font-mono text-xs text-ink" translate="no">
                        {route.path}
                      </span>
                    </div>
                    <p className="mt-2 text-sm text-ink-muted">{route.description}</p>
                  </div>
                ))}
              </div>
            </div>
          </section>

          <section className="grid gap-4 md:grid-cols-2">
            <TaxonomyPanel title="Task status values" values={taskStatuses} />
            <TaxonomyPanel title="Validation severities" values={severities} />
          </section>
        </main>
      </div>
    </div>
  );
}

function SummaryMetric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded border border-slate-200 bg-surface-raised p-4">
      <div className="text-sm font-medium text-ink-muted">{label}</div>
      <div className="mt-3 text-3xl font-semibold tracking-normal" translate="no">
        {value}
      </div>
    </div>
  );
}

function TaxonomyPanel({ title, values }: { title: string; values: string[] }) {
  return (
    <div className="rounded border border-slate-200 bg-surface-raised p-4">
      <h2 className="text-sm font-semibold">{title}</h2>
      <div className="mt-3 flex flex-wrap gap-2">
        {values.map((value) => (
          <span className="rounded bg-surface-muted px-2 py-1 font-mono text-xs text-ink-muted" key={value} translate="no">
            {value}
          </span>
        ))}
      </div>
    </div>
  );
}
