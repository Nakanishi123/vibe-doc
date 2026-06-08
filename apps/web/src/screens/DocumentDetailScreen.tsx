import { useEffect, useState } from "react";

import type { DocumentDetail, DocumentSummary, ValidationIssue } from "../lib/api-types";
import type { LoadState, Navigate } from "../lib/app-types";
import { loadJson } from "../lib/api";
import { issuesForDocument } from "../lib/documents";
import { LoadBoundary, Panel } from "../components/chrome";
import { KindBadge } from "../components/document-badges";
import { IssueList } from "../components/issue-list";
import { MarkdownView } from "../components/markdown-view";
import { RelatedList } from "../components/related-list";

export function DocumentDetailScreen({
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
    void loadJson<DocumentDetail>(`/api/documents/${id}`).then(setDetail);
  }, [id]);

  if (detail.status !== "ready") {
    return <LoadBoundary state={detail} />;
  }

  const document = detail.data;
  const issues = issuesForDocument(validationIssues, document);

  return (
    <article className="space-y-5">
      <div className="flex flex-col gap-3 border-b border-line pb-5 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0">
          <div className="mb-2 flex flex-wrap items-center gap-2 text-sm text-ink-muted">
            <KindBadge kind={document.kind} />
            <span translate="no">#{document.id}</span>
            <span className="truncate" translate="no">{document.path}</span>
          </div>
          <h2 className="font-display text-3xl font-normal tracking-normal">{document.title}</h2>
        </div>
        <button
          className="w-fit rounded border border-line bg-surface-raised px-3 py-2 text-sm font-medium text-ink-muted hover:bg-surface-muted hover:text-ink"
          onClick={() => navigate({ name: "documents" }, "/documents")}
          type="button"
        >
          Back to list
        </button>
      </div>

      <section className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_320px]">
        <div className="space-y-5">
          <Panel title="Rendered Markdown">
            <MarkdownView markdown={document.markdown} />
          </Panel>
          <Panel title="Raw Markdown">
            <pre className="max-h-[520px] overflow-auto whitespace-pre-wrap rounded bg-slate-950 p-4 text-sm text-slate-100" translate="no">
              {document.markdown}
            </pre>
          </Panel>
        </div>

        <aside className="space-y-5">
          <Panel title="Frontmatter">
            <pre className="max-h-96 overflow-auto whitespace-pre-wrap rounded bg-surface-muted p-3 text-xs text-ink-muted" translate="no">
              {JSON.stringify(document.frontmatter, null, 2)}
            </pre>
          </Panel>
          <Panel title="Related IDs">
            <RelatedList detail={document} documents={documents} navigate={navigate} />
          </Panel>
          <Panel title="Validation">
            <IssueList issues={[...document.validation, ...issues]} />
          </Panel>
        </aside>
      </section>
    </article>
  );
}
