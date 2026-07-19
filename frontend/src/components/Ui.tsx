import type { ButtonHTMLAttributes, ReactNode } from "react";
import type { DocumentSummary } from "../types";
import { navigate } from "../routes/navigation";

export function LinkButton({
  to,
  children,
  className = "",
}: {
  to: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <button className={`link-button ${className}`} onClick={() => navigate(to)} type="button">
      {children}
    </button>
  );
}

export function Tag({ name }: { name: string }) {
  return (
    <LinkButton className="tag" to={`/tag/${encodeURIComponent(name)}`}>
      <span>#</span>
      {name}
    </LinkButton>
  );
}

export function KindMark({ kind }: { kind?: string }) {
  const initial = kind === "architecture" ? "A" : kind === "decision" ? "D" : "T";
  return <span className={`kind-mark kind-${kind ?? "unknown"}`}>{initial}</span>;
}

export function Status({ value }: { value?: string }) {
  if (!value) return null;
  return <span className={`status status-${value}`}>{value}</span>;
}

export function DocumentCard({ document, index }: { document: DocumentSummary; index?: number }) {
  return (
    <article
      className="document-card"
      style={{ "--delay": `${(index ?? 0) * 35}ms` } as React.CSSProperties}
    >
      <button
        className="document-card-main"
        onClick={() => navigate(`/documents/${encodeURIComponent(document.id)}`)}
        type="button"
      >
        <KindMark kind={document.kind} />
        <span className="document-card-copy">
          <span className="eyebrow-row">
            <strong>{document.id}</strong>
            <Status value={document.status} />
          </span>
          <span className="document-title">{document.title}</span>
          <span className="document-path">{document.path}</span>
        </span>
        <span aria-hidden="true" className="arrow">
          ↗
        </span>
      </button>
      {document.tags.length > 0 && (
        <div className="tag-row">
          {document.tags.map((tag) => (
            <Tag key={tag} name={tag} />
          ))}
        </div>
      )}
    </article>
  );
}

export function PageHeader({
  kicker,
  title,
  description,
  aside,
}: {
  kicker: string;
  title: string;
  description: string;
  aside?: ReactNode;
}) {
  return (
    <header className="page-header">
      <div>
        <p className="kicker">{kicker}</p>
        <h1>{title}</h1>
        <p className="page-description">{description}</p>
      </div>
      {aside && <div className="page-header-aside">{aside}</div>}
    </header>
  );
}

export function EmptyState({ children }: { children: ReactNode }) {
  return (
    <div className="empty-state">
      <span>◇</span>
      <p>{children}</p>
    </div>
  );
}

export function Loading() {
  return (
    <div className="loading">
      <span />
      <span />
      <span />
      <em>Indexing documents</em>
    </div>
  );
}

export function ErrorState({ error }: { error: Error }) {
  return (
    <div className="error-state">
      <strong>Couldn’t load this view.</strong>
      <p>{error.message}</p>
    </div>
  );
}

export function Icon({ name }: { name: string }) {
  const paths: Record<string, ReactNode> = {
    dashboard: (
      <>
        <rect x="3" y="3" width="7" height="7" rx="1" />
        <rect x="14" y="3" width="7" height="7" rx="1" />
        <rect x="3" y="14" width="7" height="7" rx="1" />
        <rect x="14" y="14" width="7" height="7" rx="1" />
      </>
    ),
    documents: (
      <>
        <path d="M6 2h9l4 4v16H6z" />
        <path d="M14 2v5h5M9 12h6M9 16h6" />
      </>
    ),
    decisions: (
      <>
        <path d="M12 3v18M5 6h14M5 18h14" />
        <circle cx="5" cy="12" r="3" />
        <circle cx="19" cy="12" r="3" />
      </>
    ),
    tasks: (
      <>
        <rect x="3" y="3" width="18" height="18" rx="2" />
        <path d="m7 12 3 3 7-7" />
      </>
    ),
    tags: (
      <>
        <path d="M20 13 13 20 3 10V3h7z" />
        <circle cx="7.5" cy="7.5" r="1" />
      </>
    ),
    links: (
      <>
        <path d="M10 13a5 5 0 0 0 7.5.5l2-2a5 5 0 0 0-7-7l-1.2 1.2M14 11a5 5 0 0 0-7.5-.5l-2 2a5 5 0 0 0 7 7l1.2-1.2" />
      </>
    ),
    lint: (
      <>
        <path d="m12 3 10 18H2z" />
        <path d="M12 9v5M12 18h.01" />
      </>
    ),
    search: (
      <>
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-4-4" />
      </>
    ),
  };
  return (
    <svg
      aria-hidden="true"
      className="icon"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.7"
      viewBox="0 0 24 24"
    >
      {paths[name]}
    </svg>
  );
}

export function Button(props: ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button {...props} className={`button ${props.className ?? ""}`} />;
}
