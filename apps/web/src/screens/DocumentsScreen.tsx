import { useMemo, useState } from "react";

import type { DocumentSummary, ValidationIssue } from "../lib/api-types";
import type { LoadState, Navigate } from "../lib/app-types";
import { initialFilters } from "../lib/app-types";
import { filterDocuments } from "../lib/documents";
import { LoadBoundary, ScreenHeading } from "../components/chrome";
import { DocumentFilters } from "../components/document-filters";
import { DocumentTable } from "../components/document-table";

export function DocumentsScreen({
  documents,
  navigate,
  validationIssues,
}: {
  documents: LoadState<DocumentSummary[]>;
  navigate: Navigate;
  validationIssues: ValidationIssue[];
}) {
  const [filters, setFilters] = useState(initialFilters);
  const filtered = useMemo(
    () => (documents.status === "ready" ? filterDocuments(documents.data, filters) : []),
    [documents, filters],
  );

  return (
    <div className="space-y-5">
      <ScreenHeading
        eyebrow="Documents"
        title="Browse repository documents"
        meta={documents.status === "ready" ? `${filtered.length} of ${documents.data.length}` : "Loading"}
      />
      <DocumentFilters filters={filters} onChange={setFilters} />
      <LoadBoundary state={documents}>
        <DocumentTable
          documents={filtered}
          emptyMessage="No documents match these filters."
          navigate={navigate}
          validationIssues={validationIssues}
        />
      </LoadBoundary>
    </div>
  );
}
