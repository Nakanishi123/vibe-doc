import { useState } from "react";
import { api } from "../api/client";
import { useApi } from "../api/useApi";
import { EmptyState, ErrorState, Loading, PageHeader } from "../components/Ui";

export function LintPage() {
  const [filter, setFilter] = useState("");
  const { data, error } = useApi(() => api.lint(), []);
  if (error)
    return (
      <div className="page">
        <ErrorState error={error} />
      </div>
    );
  if (!data)
    return (
      <div className="page">
        <Loading />
      </div>
    );
  const diagnostics = data.diagnostics.filter((item) => !filter || item.level === filter);
  return (
    <div className="page">
      <PageHeader
        kicker="Repository health"
        title="Lint diagnostics"
        description="Structural problems are visible here; source documents remain untouched."
        aside={
          <div className={`lint-seal ${data.errors === 0 ? "clean" : "unclean"}`}>
            <strong>{data.errors === 0 ? "PASS" : "CHECK"}</strong>
            <span>
              {data.errors} errors · {data.warnings} warnings
            </span>
          </div>
        }
      />
      <section className="lint-summary">
        <button aria-pressed={!filter} onClick={() => setFilter("")} type="button">
          <span>All findings</span>
          <strong>{data.diagnostics.length}</strong>
        </button>
        <button aria-pressed={filter === "error"} onClick={() => setFilter("error")} type="button">
          <span>
            <i className="error-dot" /> Errors
          </span>
          <strong>{data.errors}</strong>
        </button>
        <button
          aria-pressed={filter === "warning"}
          onClick={() => setFilter("warning")}
          type="button"
        >
          <span>
            <i className="warning-dot" /> Warnings
          </span>
          <strong>{data.warnings}</strong>
        </button>
      </section>
      {diagnostics.length === 0 ? (
        <EmptyState>
          {data.diagnostics.length === 0
            ? "The document tree is clear. No lint findings."
            : "No findings at this severity."}
        </EmptyState>
      ) : (
        <div className="diagnostic-list">
          {diagnostics.map((item, index) => (
            <article key={`${item.path}-${index}`}>
              <span className={`diagnostic-level ${item.level}`}>{item.level}</span>
              <div>
                <strong>{item.message}</strong>
                <code>{item.path}</code>
              </div>
              <span className="diagnostic-number">{String(index + 1).padStart(2, "0")}</span>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}
