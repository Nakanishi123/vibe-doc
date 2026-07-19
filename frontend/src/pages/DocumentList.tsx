import { useEffect, useState } from "react";
import { api } from "../api/client";
import { useApi } from "../api/useApi";
import { DocumentCard, EmptyState, ErrorState, Icon, Loading, PageHeader } from "../components/Ui";

const taskStatuses = ["", "todo", "in-progress", "done", "wont-do"];

export function DocumentList({ mode }: { mode: "documents" | "decisions" | "tasks" }) {
  const searchParams = new URLSearchParams(window.location.search);
  const [query, setQuery] = useState(searchParams.get("q") ?? "");
  const [kind, setKind] = useState(
    mode === "documents"
      ? (searchParams.get("kind") ?? "")
      : mode === "decisions"
        ? "decision"
        : "task",
  );
  const [status, setStatus] = useState(searchParams.get("status") ?? "");
  const [tag, setTag] = useState(searchParams.get("tag") ?? "");
  const [settledQuery, setSettledQuery] = useState(query);
  useEffect(() => {
    const timeout = window.setTimeout(() => setSettledQuery(query), 220);
    return () => window.clearTimeout(timeout);
  }, [query]);
  const { data, error } = useApi(
    () => api.documents({ q: settledQuery, kind, status, tag }),
    [settledQuery, kind, status, tag],
  );
  const tags = useApi(() => api.tags(), []);
  const copy =
    mode === "decisions"
      ? [
          "Decision archive",
          "Decisions",
          "Trace each choice back to its context, consequence, and current status.",
        ]
      : mode === "tasks"
        ? [
            "Execution ledger",
            "Tasks",
            "Follow work from the first intention through completion—or a deliberate stop.",
          ]
        : [
            "Knowledge index",
            "Documents",
            "Search every title, identifier, tag, and paragraph in the repository.",
          ];

  return (
    <div className="page">
      <PageHeader
        kicker={copy[0]}
        title={copy[1]}
        description={copy[2]}
        aside={
          <span className="result-count">
            <strong>{data?.length ?? "—"}</strong> results
          </span>
        }
      />
      <section className="filter-panel">
        <label className="search-field">
          <Icon name="search" />
          <input
            aria-label="Search documents"
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search ID, title, tag, or body…"
            type="search"
            value={query}
          />
          <kbd>⌘ K</kbd>
        </label>
        {mode === "tasks" ? (
          <div className="status-tabs" role="tablist">
            {taskStatuses.map((value) => (
              <button
                aria-selected={status === value}
                key={value || "all"}
                onClick={() => setStatus(value)}
                role="tab"
                type="button"
              >
                {value || "all"}
              </button>
            ))}
          </div>
        ) : (
          <div className="select-row">
            {mode === "documents" && (
              <label>
                <span>Kind</span>
                <select onChange={(event) => setKind(event.target.value)} value={kind}>
                  <option value="">All kinds</option>
                  <option value="architecture">Architecture</option>
                  <option value="decision">Decision</option>
                  <option value="task">Task</option>
                </select>
              </label>
            )}
            <label>
              <span>Status</span>
              <select onChange={(event) => setStatus(event.target.value)} value={status}>
                <option value="">All statuses</option>
                <option value="accepted">Accepted</option>
                <option value="proposed">Proposed</option>
                <option value="todo">Todo</option>
                <option value="in-progress">In progress</option>
                <option value="done">Done</option>
                <option value="wont-do">Won’t do</option>
              </select>
            </label>
            <label>
              <span>Tag</span>
              <select onChange={(event) => setTag(event.target.value)} value={tag}>
                <option value="">All tags</option>
                {tags.data?.map((item) => (
                  <option key={item.name} value={item.name}>
                    {item.name}
                  </option>
                ))}
              </select>
            </label>
          </div>
        )}
      </section>
      {error ? (
        <ErrorState error={error} />
      ) : !data ? (
        <Loading />
      ) : data.length === 0 ? (
        <EmptyState>No documents match these filters.</EmptyState>
      ) : (
        <div className="document-list roomy">
          {data.map((doc, index) => (
            <DocumentCard document={doc} index={index} key={doc.id} />
          ))}
        </div>
      )}
    </div>
  );
}
