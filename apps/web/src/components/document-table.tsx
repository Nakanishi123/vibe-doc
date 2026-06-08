import type { DocumentSummary, ValidationIssue } from "../lib/api-types";
import type { Navigate } from "../lib/app-types";
import { issuesForDocument } from "../lib/documents";
import { IdChips, KindBadge, TagList } from "./document-badges";

export function DocumentTable({
  documents,
  emptyMessage,
  navigate,
  validationIssues,
}: {
  documents: DocumentSummary[];
  emptyMessage: string;
  navigate: Navigate;
  validationIssues: ValidationIssue[];
}) {
  return (
    <div className="overflow-hidden rounded border border-slate-200 bg-surface-raised">
      <table className="w-full table-fixed text-left text-sm">
        <thead className="bg-surface-muted text-xs uppercase text-ink-soft">
          <tr>
            <th className="w-20 px-4 py-3">ID</th>
            <th className="px-4 py-3">Title</th>
            <th className="hidden w-32 px-4 py-3 md:table-cell">Kind</th>
            <th className="hidden w-52 px-4 py-3 lg:table-cell">Tags</th>
            <th className="w-24 px-4 py-3">Issues</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-200">
          {documents.length === 0 ? (
            <tr>
              <td className="px-4 py-8 text-center text-ink-muted" colSpan={5}>{emptyMessage}</td>
            </tr>
          ) : (
            documents.map((document) => (
              <tr className="align-top hover:bg-surface-muted" key={document.id}>
                <td className="px-4 py-3 font-medium" translate="no">#{document.id}</td>
                <td className="min-w-0 px-4 py-3">
                  <button
                    className="block max-w-full truncate text-left font-medium text-action hover:underline"
                    onClick={() => navigate({ name: "detail", id: document.id }, `/documents/${document.id}`)}
                    type="button"
                  >
                    {document.title}
                  </button>
                  <div className="truncate text-xs text-ink-soft" translate="no">{document.path}</div>
                </td>
                <td className="hidden px-4 py-3 md:table-cell"><KindBadge kind={document.kind} /></td>
                <td className="hidden px-4 py-3 lg:table-cell"><TagList tags={document.tags ?? []} /></td>
                <td className="px-4 py-3" translate="no">{issuesForDocument(validationIssues, document).length}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

export function SpecializedTable<T extends DocumentSummary>({
  documents,
  emptyMessage,
  navigate,
  relationColumns,
  validationIssues,
}: {
  documents: T[];
  emptyMessage: string;
  navigate: Navigate;
  relationColumns: Array<[string, (document: T) => number[]]>;
  validationIssues: ValidationIssue[];
}) {
  return (
    <div className="overflow-hidden rounded border border-slate-200 bg-surface-raised">
      <table className="w-full table-fixed text-left text-sm">
        <thead className="bg-surface-muted text-xs uppercase text-ink-soft">
          <tr>
            <th className="w-20 px-4 py-3">ID</th>
            <th className="px-4 py-3">Title</th>
            {relationColumns.map(([label]) => (
              <th className="hidden w-32 px-4 py-3 md:table-cell" key={label}>{label}</th>
            ))}
            <th className="w-24 px-4 py-3">Issues</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-200">
          {documents.length === 0 ? (
            <tr>
              <td className="px-4 py-8 text-center text-ink-muted" colSpan={3 + relationColumns.length}>{emptyMessage}</td>
            </tr>
          ) : (
            documents.map((document) => (
              <tr className="align-top hover:bg-surface-muted" key={document.id}>
                <td className="px-4 py-3 font-medium" translate="no">#{document.id}</td>
                <td className="min-w-0 px-4 py-3">
                  <button
                    className="block max-w-full truncate text-left font-medium text-action hover:underline"
                    onClick={() => navigate({ name: "detail", id: document.id }, `/documents/${document.id}`)}
                    type="button"
                  >
                    {document.title}
                  </button>
                  <div className="mt-2"><TagList tags={document.tags ?? []} /></div>
                </td>
                {relationColumns.map(([label, selectIds]) => (
                  <td className="hidden px-4 py-3 md:table-cell" key={label}>
                    <IdChips ids={selectIds(document)} navigate={navigate} />
                  </td>
                ))}
                <td className="px-4 py-3" translate="no">{issuesForDocument(validationIssues, document).length}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
