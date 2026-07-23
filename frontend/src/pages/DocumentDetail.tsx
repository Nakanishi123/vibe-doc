import { api } from "../api/client";
import { useApi } from "../api/useApi";
import {
  DocumentCard,
  EmptyState,
  ErrorState,
  KindMark,
  LinkButton,
  Loading,
  Status,
  Tag,
} from "../components/Ui";
import { MarkdownBody } from "../markdown/MarkdownBody";
import type { DocumentSummary } from "../types";

function RelationGroup({
  title,
  label,
  documents,
}: {
  title: string;
  label: string;
  documents: DocumentSummary[];
}) {
  if (documents.length === 0) return null;
  return (
    <section className="relation-group">
      <p className="kicker">{label}</p>
      <h2>{title}</h2>
      <div className="document-list compact">
        {documents.map((doc) => (
          <DocumentCard document={doc} key={doc.id} />
        ))}
      </div>
    </section>
  );
}

function DocumentNavigationLink({
  direction,
  document,
}: {
  direction: "previous" | "next";
  document: DocumentSummary;
}) {
  const isPrevious = direction === "previous";
  return (
    <LinkButton
      className={`document-navigation-link document-navigation-${direction}`}
      to={`/documents/${encodeURIComponent(document.id)}`}
    >
      <span className="document-navigation-label">{isPrevious ? "← 前へ" : "次へ →"}</span>
      <strong>{document.id}</strong>
      <span className="document-navigation-title">{document.title}</span>
    </LinkButton>
  );
}

export function DocumentDetail({ id }: { id: string }) {
  const { data, error } = useApi(() => api.document(id), [id]);
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
  const relationCount =
    data.related.length +
    data.dependencies.length +
    data.dependents.length +
    data.backlinks.length +
    data.bodyLinks.length;
  return (
    <div className="detail-page">
      <header className="detail-hero">
        <LinkButton className="back-link" to="/documents">
          ← Document index
        </LinkButton>
        <div className="detail-title-row">
          <KindMark kind={data.kind} />
          <div>
            <div className="eyebrow-row">
              <strong>{data.id}</strong>
              <Status value={data.status} />
              <span className="read-only-pill">READ ONLY</span>
            </div>
            <h1>{data.title}</h1>
          </div>
        </div>
        <div className="detail-tags">
          {data.tags.map((tag) => (
            <Tag key={tag} name={tag} />
          ))}
        </div>
      </header>
      <div className="detail-grid">
        <div className="document-column">
          <article className="document-paper">
            <MarkdownBody linkedDocuments={data.bodyLinks} source={data.body} />
          </article>
          {(data.previousDocument || data.nextDocument) && (
            <nav className="document-navigation" aria-label="前後の文書">
              {data.previousDocument && (
                <DocumentNavigationLink direction="previous" document={data.previousDocument} />
              )}
              {data.nextDocument && (
                <DocumentNavigationLink direction="next" document={data.nextDocument} />
              )}
            </nav>
          )}
        </div>
        <aside className="metadata-rail">
          <section>
            <p className="kicker">Front matter</p>
            <h2>Document data</h2>
            <dl>
              <div>
                <dt>Kind</dt>
                <dd>{data.kind ?? "—"}</dd>
              </div>
              <div>
                <dt>Status</dt>
                <dd>{data.status ?? "—"}</dd>
              </div>
              <div>
                <dt>Schema</dt>
                <dd>{data.schemaVersion ?? "—"}</dd>
              </div>
              <div>
                <dt>Path</dt>
                <dd className="path-value">{data.path}</dd>
              </div>
              {Object.entries(data.extra).map(([key, value]) => (
                <div key={key}>
                  <dt>{key.replaceAll("_", " ")}</dt>
                  <dd>{value}</dd>
                </div>
              ))}
            </dl>
          </section>
          <section className="connection-count">
            <span>Connections</span>
            <strong>{String(relationCount).padStart(2, "0")}</strong>
            <small>indexed relationships</small>
          </section>
        </aside>
      </div>
      <section className="relations-section">
        <div className="section-heading">
          <div>
            <p className="kicker">Cross-reference</p>
            <h2>Connected documents</h2>
          </div>
        </div>
        {relationCount === 0 ? (
          <EmptyState>No indexed relationships for this document.</EmptyState>
        ) : (
          <div className="relation-grid">
            <RelationGroup title="Related" label="Symmetric" documents={data.related} />
            <RelationGroup title="Dependencies" label="Depends on" documents={data.dependencies} />
            <RelationGroup
              title="Dependent tasks"
              label="Required by"
              documents={data.dependents}
            />
            <RelationGroup title="References" label="Backlinks" documents={data.backlinks} />
            <RelationGroup title="Body links" label="Markdown" documents={data.bodyLinks} />
          </div>
        )}
      </section>
    </div>
  );
}
