import { useState } from "react";
import { api } from "../api/client";
import { useApi } from "../api/useApi";
import {
  EmptyState,
  ErrorState,
  KindMark,
  LinkButton,
  Loading,
  PageHeader,
} from "../components/Ui";

const labels = { related: "Related", "depends-on": "Depends on", "markdown-link": "Markdown link" };

export function Links() {
  const [filter, setFilter] = useState("");
  const { data, error } = useApi(() => api.links(), []);
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
  const visible = data.filter((link) => !filter || link.relation === filter);
  return (
    <div className="page">
      <PageHeader
        kicker="Relationship map"
        title="Links"
        description="Follow explicit relationships, task dependencies, and references written into Markdown."
        aside={
          <span className="result-count">
            <strong>{data.length}</strong> edges
          </span>
        }
      />
      <div className="status-tabs link-tabs">
        {["", "related", "depends-on", "markdown-link"].map((value) => (
          <button
            aria-selected={filter === value}
            key={value || "all"}
            onClick={() => setFilter(value)}
            type="button"
          >
            {value ? labels[value as keyof typeof labels] : "All"}
          </button>
        ))}
      </div>
      {visible.length === 0 ? (
        <EmptyState>No relationships of this type.</EmptyState>
      ) : (
        <div className="link-list">
          {visible.map((link, index) => (
            <article
              className="link-edge"
              key={`${link.source.id}-${link.target.id}-${link.relation}-${index}`}
            >
              <span className={`relation-badge relation-${link.relation}`}>
                {labels[link.relation]}
              </span>
              <LinkButton to={`/documents/${encodeURIComponent(link.source.id)}`}>
                <KindMark kind={link.source.kind} />
                <span>
                  <strong>{link.source.id}</strong>
                  <small>{link.source.title}</small>
                </span>
              </LinkButton>
              <div className="edge-line">
                <i />
                <span>→</span>
              </div>
              <LinkButton to={`/documents/${encodeURIComponent(link.target.id)}`}>
                <KindMark kind={link.target.kind} />
                <span>
                  <strong>{link.target.id}</strong>
                  <small>{link.target.title}</small>
                </span>
              </LinkButton>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}
