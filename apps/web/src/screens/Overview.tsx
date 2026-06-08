import type { DocumentSummary, HealthResponse, ValidationResponse } from "../lib/api-types";
import type { LoadState, Navigate } from "../lib/app-types";
import { countByKind, validationLabel } from "../lib/documents";
import { KeyValue, Panel, SummaryMetric } from "../components/chrome";
import { DocumentTable } from "../components/document-table";

export function Overview({
  documents,
  health,
  navigate,
  validation,
}: {
  documents: LoadState<DocumentSummary[]>;
  health: LoadState<HealthResponse>;
  navigate: Navigate;
  validation: LoadState<ValidationResponse>;
}) {
  const readyDocuments = documents.status === "ready" ? documents.data : [];
  const counts = countByKind(readyDocuments);
  const recentDocuments = readyDocuments.slice(-6).reverse();
  const validationData = validation.status === "ready" ? validation.data : null;

  return (
    <div className="space-y-6">
      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4" aria-label="Repository summary">
        <SummaryMetric label="Documents" value={health.status === "ready" ? health.data.document_count : readyDocuments.length} />
        <SummaryMetric label="Specs" value={counts.spec} />
        <SummaryMetric label="Designs" value={counts.design} />
        <SummaryMetric label="Validation errors" value={validationData?.error_count ?? 0} />
      </section>

      <section className="grid gap-6 xl:grid-cols-[1fr_340px]">
        <Panel title="Recent documents">
          <DocumentTable
            documents={recentDocuments}
            emptyMessage="No documents found."
            navigate={navigate}
            validationIssues={validationData?.issues ?? []}
          />
        </Panel>
        <Panel title="Repository">
          <div className="space-y-3 text-sm">
            <KeyValue label="Root" value={health.status === "ready" ? health.data.repository_root : "Loading"} />
            <KeyValue label="Validation" value={validationLabel(validation)} />
            <KeyValue label="Warnings" value={String(validationData?.warning_count ?? 0)} />
          </div>
        </Panel>
      </section>
    </div>
  );
}
