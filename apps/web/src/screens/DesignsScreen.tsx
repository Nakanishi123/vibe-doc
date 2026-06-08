import type { DesignSummary, ValidationIssue } from "../lib/api-types";
import type { LoadState, Navigate } from "../lib/app-types";
import { LoadBoundary, ScreenHeading } from "../components/chrome";
import { SpecializedTable } from "../components/document-table";

export function DesignsScreen({
  designs,
  navigate,
  validationIssues,
}: {
  designs: LoadState<DesignSummary[]>;
  navigate: Navigate;
  validationIssues: ValidationIssue[];
}) {
  return (
    <div className="space-y-5">
      <ScreenHeading
        eyebrow="Designs"
        title="Design documents"
        meta={designs.status === "ready" ? `${designs.data.length} designs` : "Loading"}
      />
      <LoadBoundary state={designs}>
        <SpecializedTable
          documents={designs.status === "ready" ? designs.data : []}
          emptyMessage="No designs found."
          navigate={navigate}
          relationColumns={[
            ["Specs", (document) => document.specs],
            ["ADRs", (document) => document.adrs],
            ["Tasks", (document) => document.related_tasks],
          ]}
          validationIssues={validationIssues}
        />
      </LoadBoundary>
    </div>
  );
}
