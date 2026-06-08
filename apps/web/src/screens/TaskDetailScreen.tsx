import { useEffect, useMemo, useState } from "react";

import { IdChips, KindBadge, TagList } from "../components/document-badges";
import { IssueList } from "../components/issue-list";
import { MarkdownView } from "../components/markdown-view";
import { RelatedList } from "../components/related-list";
import { KeyValue, LoadBoundary, Panel } from "../components/chrome";
import { loadJson } from "../lib/api";
import type { DocumentDetail, DocumentSummary, ValidationIssue } from "../lib/api-types";
import type { LoadState, Navigate } from "../lib/app-types";
import { issuesForDocument } from "../lib/documents";

export function TaskDetailScreen({
  documents,
  id,
  navigate,
  validationIssues,
}: {
  documents: DocumentSummary[];
  id: number;
  navigate: Navigate;
  validationIssues: ValidationIssue[];
}) {
  const [detail, setDetail] = useState<LoadState<DocumentDetail>>({ status: "loading" });

  useEffect(() => {
    setDetail({ status: "loading" });
    void loadJson<DocumentDetail>(`/api/tasks/${id}`).then(setDetail);
  }, [id]);

  if (detail.status !== "ready") {
    return <LoadBoundary state={detail} />;
  }

  const task = detail.data;
  const issues = issuesForDocument(validationIssues, task);
  const frontmatter = task.frontmatter;

  return (
    <article className="space-y-5">
      <div className="flex flex-col gap-3 border-b border-slate-200 pb-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0">
          <div className="mb-2 flex flex-wrap items-center gap-2 text-sm text-ink-muted">
            <KindBadge kind={task.kind} />
            <span translate="no">#{task.id}</span>
            <span className="truncate" translate="no">{task.path}</span>
          </div>
          <h2 className="text-2xl font-semibold tracking-normal">{task.title}</h2>
          <div className="mt-3"><TagList tags={task.tags ?? []} /></div>
        </div>
        <button
          className="w-fit rounded border border-slate-200 bg-surface-raised px-3 py-2 text-sm font-medium text-ink-muted hover:bg-surface-muted hover:text-ink"
          onClick={() => navigate({ name: "tasks" }, "/tasks")}
          type="button"
        >
          Back to tasks
        </button>
      </div>

      <section className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_340px]">
        <div className="space-y-5">
          <Panel title="Task body">
            <MarkdownView markdown={task.markdown} />
          </Panel>
          <Panel title="Dependencies">
            <DependencyList detail={task} documents={documents} navigate={navigate} />
          </Panel>
        </div>

        <aside className="space-y-5">
          <Panel title="Task metadata">
            <TaskMetadata frontmatter={frontmatter} />
          </Panel>
          <Panel title="Related documents">
            <RelatedList detail={task} documents={documents} navigate={navigate} />
          </Panel>
          <Panel title="Validation">
            <IssueList issues={[...task.validation, ...issues]} />
          </Panel>
        </aside>
      </section>
    </article>
  );
}

function TaskMetadata({ frontmatter }: { frontmatter: Record<string, unknown> }) {
  const rows = [
    ["Status", frontmatter.status],
    ["Type", frontmatter.type],
    ["Priority", frontmatter.priority ?? "medium"],
    ["Specs", frontmatter.specs],
    ["Designs", frontmatter.designs],
    ["ADRs", frontmatter.adrs],
    ["Depends on", frontmatter.depends_on],
  ];

  return (
    <dl className="grid gap-3">
      {rows.map(([label, value]) => (
        <KeyValue key={label as string} label={label as string} value={metadataValue(value)} />
      ))}
    </dl>
  );
}

function DependencyList({
  detail,
  documents,
  navigate,
}: {
  detail: DocumentDetail;
  documents: DocumentSummary[];
  navigate: Navigate;
}) {
  const dependencies = useMemo(
    () => detail.related_ids.filter((related) => related.relation === "dependency"),
    [detail.related_ids],
  );

  if (dependencies.length === 0) {
    return <p className="text-sm text-ink-muted">No task dependencies.</p>;
  }

  return (
    <div className="grid gap-3">
      {dependencies.map((dependency) => {
        const found = detail.related_documents.find((document) => document.id === dependency.id)
          ?? documents.find((document) => document.id === dependency.id);
        return (
          <div className="rounded border border-slate-200 bg-white p-3" key={dependency.id}>
            <div className="mb-2 text-xs font-medium uppercase text-ink-soft">Dependency</div>
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <span className="min-w-0 truncate text-sm font-medium">
                <span translate="no">#{dependency.id}</span> {found?.title ?? "Unresolved task"}
              </span>
              <IdChips ids={[dependency.id]} navigate={navigate} />
            </div>
          </div>
        );
      })}
    </div>
  );
}

function metadataValue(value: unknown) {
  if (Array.isArray(value)) {
    return value.length === 0 ? "[]" : value.join(", ");
  }
  if (value === null || value === undefined) {
    return "none";
  }
  return String(value);
}
