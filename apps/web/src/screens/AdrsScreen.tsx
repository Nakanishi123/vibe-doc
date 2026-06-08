import { IdChips, TagList } from "../components/document-badges";
import { LoadBoundary, ScreenHeading } from "../components/chrome";
import type { AdrSummary, ValidationIssue } from "../lib/api-types";
import type { LoadState, Navigate } from "../lib/app-types";
import { issuesForDocument } from "../lib/documents";

export function AdrsScreen({
  adrs,
  navigate,
  validationIssues,
}: {
  adrs: LoadState<AdrSummary[]>;
  navigate: Navigate;
  validationIssues: ValidationIssue[];
}) {
  return (
    <div className="space-y-5">
      <ScreenHeading
        eyebrow="ADRs"
        title="Architecture decision records"
        meta={adrs.status === "ready" ? `${adrs.data.length} ADRs` : "Loading"}
      />
      <LoadBoundary state={adrs}>
        <div className="overflow-hidden rounded-lg border border-line bg-surface-raised">
          <table className="w-full table-fixed text-left text-sm">
            <thead className="bg-surface-muted text-xs uppercase text-ink-soft">
              <tr>
                <th className="w-20 px-4 py-3">ID</th>
                <th className="px-4 py-3">Decision</th>
                <th className="hidden w-32 px-4 py-3 md:table-cell">Status</th>
                <th className="hidden w-32 px-4 py-3 lg:table-cell">Date</th>
                <th className="hidden w-36 px-4 py-3 xl:table-cell">Supersedes</th>
                <th className="hidden w-36 px-4 py-3 xl:table-cell">Superseded by</th>
                <th className="w-20 px-4 py-3">Issues</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-line">
              {adrs.status === "ready" && adrs.data.length === 0 ? (
                <tr>
                  <td className="px-4 py-8 text-center text-ink-muted" colSpan={7}>No ADRs found.</td>
                </tr>
              ) : null}
              {adrs.status === "ready"
                ? adrs.data.map((adr) => (
                  <tr className="align-top hover:bg-surface-muted" key={adr.id}>
                    <td className="px-4 py-3 font-medium" translate="no">#{adr.id}</td>
                    <td className="min-w-0 px-4 py-3">
                      <button
                        className="block max-w-full truncate text-left font-medium text-action hover:underline"
                        onClick={() => navigate({ name: "detail", id: adr.id }, `/documents/${adr.id}`)}
                        type="button"
                      >
                        {adr.title}
                      </button>
                      <div className="mt-2"><TagList tags={adr.tags ?? []} /></div>
                      <div className="mt-2 grid gap-1 text-xs text-ink-muted md:hidden" translate="no">
                        <span>{adr.status}</span>
                        <span>{adr.date ?? "No date"}</span>
                      </div>
                    </td>
                    <td className="hidden px-4 py-3 md:table-cell" translate="no"><AdrStatusBadge status={adr.status} /></td>
                    <td className="hidden px-4 py-3 lg:table-cell" translate="no">{adr.date ?? "None"}</td>
                    <td className="hidden px-4 py-3 xl:table-cell"><IdChips ids={adr.supersedes} navigate={navigate} /></td>
                    <td className="hidden px-4 py-3 xl:table-cell"><IdChips ids={adr.superseded_by ? [adr.superseded_by] : []} navigate={navigate} /></td>
                    <td className="px-4 py-3" translate="no">{issuesForDocument(validationIssues, adr).length}</td>
                  </tr>
                ))
                : null}
            </tbody>
          </table>
        </div>
      </LoadBoundary>
    </div>
  );
}

function AdrStatusBadge({ status }: { status: string }) {
  const className = status === "accepted"
    ? "border-action-border bg-action-soft text-action"
    : status === "rejected" || status === "deprecated" || status === "superseded"
      ? "border-line bg-surface-muted text-ink-muted"
      : "border-attention-border bg-attention-soft text-attention";

  return <span className={`rounded border px-2 py-1 text-xs font-medium ${className}`}>{status}</span>;
}
