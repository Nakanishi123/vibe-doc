import type { ValidationIssue } from "../lib/api-types";

export function IssueList({ issues }: { issues: ValidationIssue[] }) {
  if (issues.length === 0) {
    return <p className="text-sm text-ink-muted">No validation warnings.</p>;
  }

  return (
    <div className="space-y-2">
      {issues.map((issue, index) => (
        <div className="rounded border border-attention-border bg-attention-soft p-3 text-sm" key={`${issue.code}-${index}`}>
          <div className="font-semibold text-attention" translate="no">{issue.severity}: {issue.code}</div>
          <p className="mt-1 text-ink">{issue.message}</p>
          {issue.path ? <p className="mt-1 text-xs text-ink-muted" translate="no">{issue.path}</p> : null}
        </div>
      ))}
    </div>
  );
}
