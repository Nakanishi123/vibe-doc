import type { DocumentKind } from "../lib/api-types";
import type { Navigate } from "../lib/app-types";

export function KindBadge({ kind }: { kind: DocumentKind }) {
  return <span className="rounded bg-action-soft px-2 py-1 text-xs font-medium text-action" translate="no">{kind}</span>;
}

export function TagList({ tags }: { tags: string[] }) {
  if (tags.length === 0) {
    return <span className="text-xs text-ink-soft">No tags</span>;
  }
  return (
    <div className="flex flex-wrap gap-1">
      {tags.map((tag) => (
        <span
          className="rounded border border-action-border/35 bg-surface-raised px-2 py-1 text-xs text-action"
          key={tag}
          translate="no"
        >
          {tag}
        </span>
      ))}
    </div>
  );
}

export function IdChips({ ids, navigate }: { ids: number[]; navigate: Navigate }) {
  if (ids.length === 0) {
    return <span className="text-ink-soft">None</span>;
  }
  return (
    <div className="flex flex-wrap gap-1">
      {ids.map((id) => (
        <button
          className="rounded bg-surface-muted px-2 py-1 font-mono text-xs text-action hover:bg-action-soft"
          key={id}
          onClick={() => navigate({ name: "detail", id }, `/documents/${id}`)}
          type="button"
        >
          #{id}
        </button>
      ))}
    </div>
  );
}
