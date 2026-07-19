import { api } from "../api/client";
import { useApi } from "../api/useApi";
import { DocumentCard, ErrorState, LinkButton, Loading, PageHeader } from "../components/Ui";

export function Tags() {
  const { data, error } = useApi(() => api.tags(), []);
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
  const max = Math.max(...data.map((tag) => tag.count), 1);
  return (
    <div className="page">
      <PageHeader
        kicker="Shared vocabulary"
        title="Tags"
        description="Explore the language that connects architecture, decisions, and delivery."
        aside={
          <span className="result-count">
            <strong>{data.length}</strong> unique
          </span>
        }
      />
      <div className="tag-cloud">
        {data.map((tag, index) => (
          <LinkButton
            className="tag-tile"
            key={tag.name}
            to={`/tag/${encodeURIComponent(tag.name)}`}
          >
            <span className="tag-index">{String(index + 1).padStart(2, "0")}</span>
            <span className="tag-name">#{tag.name}</span>
            <span className="tag-meter">
              <i style={{ width: `${(tag.count / max) * 100}%` }} />
            </span>
            <strong>
              {tag.count}
              <small> docs</small>
            </strong>
            <span className="arrow">↗</span>
          </LinkButton>
        ))}
      </div>
    </div>
  );
}

export function TagDetailPage({ tag }: { tag: string }) {
  const { data, error } = useApi(() => api.tag(tag), [tag]);
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
  return (
    <div className="page">
      <LinkButton className="back-link" to="/tags">
        ← All tags
      </LinkButton>
      <PageHeader
        kicker="Tag collection"
        title={`#${data.name}`}
        description="Every indexed document carrying this shared term."
        aside={
          <span className="result-count">
            <strong>{data.documents.length}</strong> documents
          </span>
        }
      />
      <div className="document-list roomy">
        {data.documents.map((doc, index) => (
          <DocumentCard document={doc} index={index} key={doc.id} />
        ))}
      </div>
    </div>
  );
}
