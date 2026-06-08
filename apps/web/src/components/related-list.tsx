import type { DocumentDetail, DocumentSummary } from "../lib/api-types";
import type { Navigate } from "../lib/app-types";

export function RelatedList({
  detail,
  documents,
  navigate,
}: {
  detail: DocumentDetail;
  documents: DocumentSummary[];
  navigate: Navigate;
}) {
  if (detail.related_ids.length === 0) {
    return <p className="text-sm text-ink-muted">No related IDs.</p>;
  }

  return (
    <div className="space-y-2">
      {detail.related_ids.map((related) => {
        const found = detail.related_documents.find((document) => document.id === related.id)
          ?? documents.find((document) => document.id === related.id);
        return (
          <button
            className="block w-full rounded border border-line bg-surface-raised p-3 text-left hover:bg-surface-muted"
            key={`${related.relation}-${related.id}`}
            onClick={() => navigate({ name: "detail", id: related.id }, `/documents/${related.id}`)}
            type="button"
          >
            <span className="text-xs font-medium uppercase text-ink-soft" translate="no">{related.relation}</span>
            <span className="mt-1 block truncate text-sm font-medium">
              <span translate="no">#{related.id}</span> {found?.title ?? "Unresolved document"}
            </span>
          </button>
        );
      })}
    </div>
  );
}
