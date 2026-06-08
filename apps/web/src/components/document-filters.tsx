import type { FilterState } from "../lib/app-types";

export function DocumentFilters({
  filters,
  onChange,
}: {
  filters: FilterState;
  onChange: (filters: FilterState) => void;
}) {
  return (
    <section className="grid gap-3 rounded-lg border border-line bg-surface-raised p-4 md:grid-cols-[160px_1fr_1fr_120px]">
      <label className="space-y-1 text-sm font-medium text-ink-muted">
        <span>Kind</span>
        <select
          className="h-10 w-full rounded border border-line bg-field px-3 text-ink outline-none transition focus:border-action-border"
          onChange={(event) => onChange({ ...filters, kind: event.target.value as FilterState["kind"] })}
          value={filters.kind}
        >
          <option value="all">All</option>
          <option value="spec">Spec</option>
          <option value="design">Design</option>
          <option value="adr">ADR</option>
          <option value="task">Task</option>
          <option value="task-index">Task index</option>
        </select>
      </label>
      <TextFilter label="Tag" onChange={(tag) => onChange({ ...filters, tag })} value={filters.tag} />
      <TextFilter label="Title" onChange={(title) => onChange({ ...filters, title })} value={filters.title} />
      <TextFilter label="ID" onChange={(id) => onChange({ ...filters, id })} value={filters.id} />
    </section>
  );
}

function TextFilter({ label, onChange, value }: { label: string; onChange: (value: string) => void; value: string }) {
  return (
    <label className="space-y-1 text-sm font-medium text-ink-muted">
      <span>{label}</span>
      <input
        className="h-10 w-full rounded border border-line bg-field px-3 text-ink outline-none transition focus:border-action-border"
        onChange={(event) => onChange(event.target.value)}
        type="search"
        value={value}
      />
    </label>
  );
}
