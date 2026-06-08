import type { SpecSummary, ValidationIssue } from "../lib/api-types";
import type { LoadState, Navigate } from "../lib/app-types";
import { LoadBoundary, ScreenHeading } from "../components/chrome";
import { SpecializedTable } from "../components/document-table";

export function SpecsScreen({
  navigate,
  specs,
  validationIssues,
}: {
  navigate: Navigate;
  specs: LoadState<SpecSummary[]>;
  validationIssues: ValidationIssue[];
}) {
  return (
    <div className="space-y-5">
      <ScreenHeading
        eyebrow="Specs"
        title="Specification documents"
        meta={specs.status === "ready" ? `${specs.data.length} specs` : "Loading"}
      />
      <LoadBoundary state={specs}>
        <SpecializedTable
          documents={specs.status === "ready" ? specs.data : []}
          emptyMessage="No specs found."
          navigate={navigate}
          relationColumns={[
            ["Designs", (document) => document.related_designs],
            ["Tasks", (document) => document.related_tasks],
          ]}
          validationIssues={validationIssues}
        />
      </LoadBoundary>
    </div>
  );
}
