import type { ReactNode } from "react";

import type { HealthResponse, ValidationResponse } from "../lib/api-types";
import type { LoadState } from "../lib/app-types";
import { validationLabel } from "../lib/documents";

export function Panel({ children, title }: { children: ReactNode; title: string }) {
  return (
    <section className="rounded-lg border border-line bg-surface-raised/92 p-5 shadow-sm shadow-black/5">
      <h3 className="mb-4 text-xs font-semibold uppercase text-ink-soft">{title}</h3>
      {children}
    </section>
  );
}

export function ScreenHeading({ eyebrow, meta, title }: { eyebrow: string; meta: string; title: string }) {
  return (
    <div className="flex flex-col gap-2 border-b border-line pb-5 sm:flex-row sm:items-end sm:justify-between">
      <div>
        <p className="text-xs font-semibold uppercase text-action">{eyebrow}</p>
        <h2 className="font-display text-3xl font-normal tracking-normal">{title}</h2>
      </div>
      <span className="text-sm text-ink-muted" translate="no">{meta}</span>
    </div>
  );
}

export function SummaryMetric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg border border-line bg-surface-raised/92 p-5 shadow-sm shadow-black/5">
      <div className="text-xs font-semibold uppercase text-ink-soft">{label}</div>
      <div className="mt-4 font-display text-4xl font-normal tracking-normal" translate="no">{value}</div>
    </div>
  );
}

export function KeyValue({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid gap-1">
      <dt className="text-xs font-medium uppercase text-ink-soft">{label}</dt>
      <dd className="break-words font-mono text-xs text-ink-muted" translate="no">{value}</dd>
    </div>
  );
}

export function LoadBoundary<T>({ children, state }: { children?: ReactNode; state: LoadState<T> }) {
  if (state.status === "loading") {
    return <div className="rounded-lg border border-line bg-surface-raised p-6 text-sm text-ink-muted">Loading</div>;
  }
  if (state.status === "error") {
    return <div className="rounded-lg border border-danger-border bg-danger-soft p-6 text-sm text-danger">{state.message}</div>;
  }
  return <>{children}</>;
}

export function StatusPill({
  health,
  validation,
}: {
  health: LoadState<HealthResponse>;
  validation: LoadState<ValidationResponse>;
}) {
  const label = health.status === "ready" ? "API online" : health.status === "error" ? "API error" : "API loading";
  const validationStatus = validationLabel(validation);
  return (
    <div className="flex flex-wrap gap-2 text-sm">
      <span className="rounded-full border border-line bg-surface px-3 py-2 text-ink-muted">{label}</span>
      <span className="rounded-full border border-line bg-surface px-3 py-2 text-ink-muted" translate="no">{validationStatus}</span>
    </div>
  );
}
