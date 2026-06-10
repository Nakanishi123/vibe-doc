import { useEffect, useMemo, useState } from "react";

import { IdChips, KindBadge, TagList } from "../components/document-badges";
import { IssueList } from "../components/issue-list";
import { MarkdownView } from "../components/markdown-view";
import { RelatedList } from "../components/related-list";
import { KeyValue, LoadBoundary, Panel } from "../components/chrome";
import { loadJson, postJson } from "../lib/api";
import type {
  AgentRunDetailResponse,
  AgentRunStatus,
  DocumentDetail,
  DocumentSummary,
  PrepareAgentRunResponse,
  ValidationIssue,
} from "../lib/api-types";
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
      <div className="flex flex-col gap-3 border-b border-line pb-5 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0">
          <div className="mb-2 flex flex-wrap items-center gap-2 text-sm text-ink-muted">
            <KindBadge kind={task.kind} />
            <span translate="no">#{task.id}</span>
            <span className="truncate" translate="no">{task.path}</span>
          </div>
          <h2 className="font-display text-3xl font-normal tracking-normal">{task.title}</h2>
          <div className="mt-3"><TagList tags={task.tags ?? []} /></div>
        </div>
        <button
          className="w-fit rounded border border-line bg-surface-raised px-3 py-2 text-sm font-medium text-ink-muted hover:bg-surface-muted hover:text-ink"
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
          <Panel title="Agent run review">
            <AgentRunReviewPanel taskId={id} />
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

function AgentRunReviewPanel({ taskId }: { taskId: number }) {
  const [runIdInput, setRunIdInput] = useState("");
  const [runDetail, setRunDetail] = useState<LoadState<AgentRunDetailResponse> | null>(null);
  const [reviewDraft, setReviewDraft] = useState("");
  const [message, setMessage] = useState<PanelMessage | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);

  async function prepareRun() {
    await mutate("prepare", async () => {
      const prepared = await postJson<PrepareAgentRunResponse>(`/api/tasks/${taskId}/prepare-codex`);
      if (prepared.status === "ready") {
        const detail = await loadJson<AgentRunDetailResponse>(`/api/runs/${prepared.data.run.run_id}`);
        if (detail.status === "ready") {
          setRunIdInput(detail.data.run.run_id);
          setReviewDraft(detail.data.review);
          setRunDetail(detail);
          setMessage({ tone: "info", text: "Prepared a run and saved the prompt." });
          return;
        }
        setRunDetail(detail);
        return;
      }
      if (prepared.status === "error") {
        setRunDetail({ status: "error", message: prepared.message });
      }
    });
  }

  async function loadRun() {
    const runId = runIdInput.trim();
    if (!runId) {
      setMessage({ tone: "danger", text: "Enter a run ID first." });
      return;
    }
    await refreshRun(runId);
  }

  async function approvePrompt() {
    const runId = currentRunId(runDetail, runIdInput);
    if (!runId) return;
    await mutate("approve", async () => {
      const result = await postJson(`/api/runs/${runId}/approve-prompt`);
      if (result.status === "error") {
        setRunDetail(result);
        return;
      }
      await refreshRun(runId);
      setMessage({ tone: "info", text: "Prompt approved. Start remains a separate action." });
    });
  }

  async function startRun() {
    const runId = currentRunId(runDetail, runIdInput);
    if (!runId) return;
    await mutate("start", async () => {
      try {
        const response = await fetch(`/api/runs/${runId}/start`, {
          body: JSON.stringify({ command: "codex" }),
          headers: { Accept: "application/x-ndjson", "Content-Type": "application/json" },
          method: "POST",
        });
        if (!response.ok) {
          const payload = await response.json();
          setRunDetail({ status: "error", message: apiErrorMessage(payload, response.status) });
          return;
        }
        const streamResult = parseStartRunResult(await response.text());
        await refreshRun(runId);
        setMessage(streamResult);
      } catch (error) {
        setRunDetail({
          status: "error",
          message: error instanceof Error ? error.message : "Request failed",
        });
      }
    });
  }

  async function saveReview(generate: boolean) {
    const runId = currentRunId(runDetail, runIdInput);
    if (!runId) return;
    await mutate(generate ? "generate-review" : "save-review", async () => {
      const body = generate ? {} : { content: reviewDraft };
      const result = await postJson<AgentRunDetailResponse>(`/api/runs/${runId}/review`, body);
      setRunDetail(result);
      if (result.status === "ready") {
        setReviewDraft(result.data.review);
        setMessage({
          tone: "info",
          text: generate ? "Generated and stored review output." : "Stored review output.",
        });
      }
    });
  }

  async function decide(decision: "accept" | "reject") {
    const runId = currentRunId(runDetail, runIdInput);
    if (!runId) return;
    await mutate(decision, async () => {
      const result = await postJson<AgentRunDetailResponse>(`/api/runs/${runId}/${decision}`);
      setRunDetail(result);
      if (result.status === "ready") {
        setReviewDraft(result.data.review);
        setMessage(
          decision === "accept"
            ? { tone: "info", text: "Run accepted. Task completion is still a separate action." }
            : { tone: "danger", text: "Run rejected." },
        );
      }
    });
  }

  async function refreshRun(runId: string) {
    setRunDetail({ status: "loading" });
    const detail = await loadJson<AgentRunDetailResponse>(`/api/runs/${runId}`);
    setRunDetail(detail);
    if (detail.status === "ready") {
      setRunIdInput(detail.data.run.run_id);
      setReviewDraft(detail.data.review);
      setMessage(null);
    }
  }

  async function mutate(action: string, operation: () => Promise<void>) {
    setBusyAction(action);
    setMessage(null);
    try {
      await operation();
    } finally {
      setBusyAction(null);
    }
  }

  const detail = runDetail?.status === "ready" ? runDetail.data : null;
  const run = detail?.run;
  const canApprove = run?.status === "prepared";
  const canStart = run?.status === "prompt-approved";
  const canReview = run?.status === "succeeded" || run?.status === "accepted" || run?.status === "rejected";
  const canDecide = run?.status === "succeeded";

  return (
    <div className="space-y-4">
      <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto_auto]">
        <input
          className="min-h-10 rounded border border-line bg-field px-3 py-2 font-mono text-sm text-ink outline-none focus:border-action-border"
          onChange={(event) => setRunIdInput(event.target.value)}
          placeholder="run-41-YYYYMMDDTHHMMSSZ-001"
          type="text"
          value={runIdInput}
        />
        <button className={secondaryButtonClass} disabled={busyAction !== null} onClick={loadRun} type="button">
          Load
        </button>
        <button className={primaryButtonClass} disabled={busyAction !== null} onClick={prepareRun} type="button">
          Prepare Codex Run
        </button>
      </div>

      {message ? <PanelMessageView message={message} /> : null}
      {runDetail?.status === "loading" ? <p className="text-sm text-ink-muted">Loading run</p> : null}
      {runDetail?.status === "error" ? (
        <p className="rounded border border-danger-border bg-danger-soft px-3 py-2 text-sm text-danger">{runDetail.message}</p>
      ) : null}

      {detail ? (
        <div className="space-y-4">
          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
            <RunMetric label="Status" value={reviewStatusLabel(detail.run.status)} tone={reviewStatusTone(detail.run.status)} />
            <RunMetric label="Run ID" value={detail.run.run_id} />
            <RunMetric label="Agent" value={detail.run.agent_kind} />
            <RunMetric label="Updated" value={detail.run.updated_at} />
          </div>

          <div className="flex flex-wrap gap-2">
            <button className={secondaryButtonClass} disabled={!canApprove || busyAction !== null} onClick={approvePrompt} type="button">
              Approve Prompt
            </button>
            <button className={secondaryButtonClass} disabled={!canStart || busyAction !== null} onClick={startRun} type="button">
              Start Run
            </button>
            <button className={secondaryButtonClass} disabled={!canReview || busyAction !== null} onClick={() => saveReview(true)} type="button">
              Generate Review
            </button>
            <button className={primaryButtonClass} disabled={!canDecide || busyAction !== null} onClick={() => decide("accept")} type="button">
              Accept Run
            </button>
            <button className={dangerButtonClass} disabled={!canDecide || busyAction !== null} onClick={() => decide("reject")} type="button">
              Reject Run
            </button>
          </div>

          <ArtifactBlock content={detail.prompt} emptyLabel="No prompt captured." title="Prompt" />
          <ArtifactBlock content={detail.terminal_log} emptyLabel="No terminal log captured." title="Logs" />
          <DiffBlock diff={detail.diff} />

          <div className="grid gap-3">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <h4 className="text-xs font-semibold uppercase text-ink-soft">AI review output</h4>
              <button className={secondaryButtonClass} disabled={!canReview || busyAction !== null} onClick={() => saveReview(false)} type="button">
                Save Review
              </button>
            </div>
            <textarea
              className="min-h-36 rounded border border-line bg-field p-3 font-mono text-xs text-ink outline-none focus:border-action-border"
              onChange={(event) => setReviewDraft(event.target.value)}
              placeholder="Attach review notes after inspecting the diff."
              value={reviewDraft}
            />
          </div>
        </div>
      ) : null}
    </div>
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
          <div className="rounded border border-line bg-surface-raised p-3" key={dependency.id}>
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

type PanelMessage = {
  tone: "info" | "danger";
  text: string;
};

type StartRunEvent =
  | { event: "completed"; status: AgentRunStatus; exit_result?: { code?: number | null } | null }
  | { event: "error"; message?: string }
  | { event: "terminal"; data?: string };

function PanelMessageView({ message }: { message: PanelMessage }) {
  const toneClass = message.tone === "danger"
    ? "border-danger-border bg-danger-soft text-danger"
    : "border-action-border bg-action-soft text-action";
  return <p className={`rounded border px-3 py-2 text-sm ${toneClass}`}>{message.text}</p>;
}

function RunMetric({
  label,
  tone = "neutral",
  value,
}: {
  label: string;
  tone?: "neutral" | "good" | "warn" | "danger";
  value: string;
}) {
  const toneClass = {
    danger: "border-danger-border bg-danger-soft text-danger",
    good: "border-action-border bg-action-soft text-action",
    neutral: "border-line bg-surface text-ink-muted",
    warn: "border-attention-border bg-attention-soft text-attention",
  }[tone];
  return (
    <div className={`min-w-0 rounded border px-3 py-2 ${toneClass}`}>
      <div className="text-[0.68rem] font-semibold uppercase">{label}</div>
      <div className="mt-1 truncate font-mono text-xs" translate="no">{value}</div>
    </div>
  );
}

function ArtifactBlock({
  content,
  emptyLabel,
  title,
}: {
  content: string;
  emptyLabel: string;
  title: string;
}) {
  return (
    <div className="grid gap-2">
      <h4 className="text-xs font-semibold uppercase text-ink-soft">{title}</h4>
      <pre className="max-h-[28rem] overflow-auto rounded border border-line bg-surface p-3 text-xs leading-5 text-ink-muted" translate="no">
        <code>{content || emptyLabel}</code>
      </pre>
    </div>
  );
}

function DiffBlock({ diff }: { diff: string }) {
  if (!diff) {
    return <ArtifactBlock content="" emptyLabel="No diff captured." title="Diff" />;
  }
  return (
    <div className="grid gap-2">
      <h4 className="text-xs font-semibold uppercase text-ink-soft">Diff</h4>
      <pre className="max-h-[34rem] overflow-auto rounded border border-line bg-surface p-3 text-xs leading-5" translate="no">
        <code>
          {diff.split("\n").map((line, index) => (
            <span className={diffLineClass(line)} key={`${index}-${line.slice(0, 24)}`}>
              {line || " "}
              {"\n"}
            </span>
          ))}
        </code>
      </pre>
    </div>
  );
}

function diffLineClass(line: string) {
  if (line.startsWith("+") && !line.startsWith("+++")) {
    return "block text-action";
  }
  if (line.startsWith("-") && !line.startsWith("---")) {
    return "block text-danger";
  }
  if (line.startsWith("@@")) {
    return "block text-attention";
  }
  return "block text-ink-muted";
}

function parseStartRunResult(body: string): PanelMessage {
  const events = body
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map(parseStartRunEvent)
    .filter((event): event is StartRunEvent => event !== null);
  const error = events.find((event) => event.event === "error");
  if (error?.event === "error") {
    return {
      tone: "danger",
      text: `Run failed: ${error.message ?? "agent execution returned an error event"}`,
    };
  }

  const completed = lastCompletedStartRunEvent(events);
  if (!completed) {
    return {
      tone: "danger",
      text: "Run ended without a completion event. Review the captured logs before continuing.",
    };
  }

  if (completed.status === "succeeded") {
    return {
      tone: "info",
      text: "Run finished. Review the captured logs and diff before accepting.",
    };
  }

  const exitCode = completed.exit_result?.code;
  return {
    tone: "danger",
    text: exitCode === undefined || exitCode === null
      ? `Run ${completed.status}. Review the captured logs before continuing.`
      : `Run ${completed.status} with exit code ${exitCode}. Review the captured logs before continuing.`,
  };
}

function parseStartRunEvent(line: string): StartRunEvent | null {
  try {
    const event = JSON.parse(line) as Partial<StartRunEvent>;
    if (event.event === "completed" || event.event === "error" || event.event === "terminal") {
      return event as StartRunEvent;
    }
  } catch {
    return null;
  }
  return null;
}

function lastCompletedStartRunEvent(events: StartRunEvent[]) {
  for (let index = events.length - 1; index >= 0; index -= 1) {
    const event = events[index];
    if (event.event === "completed") {
      return event;
    }
  }
  return null;
}

function currentRunId(
  runDetail: LoadState<AgentRunDetailResponse> | null,
  runIdInput: string,
) {
  const runId = runDetail?.status === "ready" ? runDetail.data.run.run_id : runIdInput.trim();
  return runId || null;
}

function reviewStatusLabel(status: AgentRunStatus) {
  if (status === "succeeded") {
    return "pending review";
  }
  return status;
}

function reviewStatusTone(status: AgentRunStatus): "neutral" | "good" | "warn" | "danger" {
  if (status === "accepted") return "good";
  if (status === "rejected" || status === "failed" || status === "cancelled") return "danger";
  if (status === "succeeded") return "warn";
  return "neutral";
}

function apiErrorMessage(payload: unknown, status: number) {
  const maybeError = payload as { error?: { message?: string } };
  return maybeError.error?.message ?? `Request failed with HTTP ${status}`;
}

const primaryButtonClass =
  "min-h-10 rounded border border-action-border bg-action px-3 py-2 text-sm font-semibold text-white transition hover:bg-action-strong disabled:cursor-not-allowed disabled:border-line disabled:bg-surface-muted disabled:text-ink-soft";

const secondaryButtonClass =
  "min-h-10 rounded border border-line bg-surface-raised px-3 py-2 text-sm font-semibold text-ink-muted transition hover:border-action-border hover:bg-surface-muted hover:text-ink disabled:cursor-not-allowed disabled:opacity-50";

const dangerButtonClass =
  "min-h-10 rounded border border-danger-border bg-danger-soft px-3 py-2 text-sm font-semibold text-danger transition hover:bg-surface-muted disabled:cursor-not-allowed disabled:opacity-50";

function metadataValue(value: unknown) {
  if (Array.isArray(value)) {
    return value.length === 0 ? "[]" : value.join(", ");
  }
  if (value === null || value === undefined) {
    return "none";
  }
  return String(value);
}
