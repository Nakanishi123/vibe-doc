import type { TaskFilterState } from "../lib/app-types";

const statusOptions: Array<TaskFilterState["status"]> = ["all", "planned", "doing", "blocked", "done", "dropped"];
const typeOptions: Array<TaskFilterState["type"]> = ["all", "feature", "bug", "refactor", "chore", "docs", "test", "spike"];
const priorityOptions: Array<TaskFilterState["priority"]> = ["all", "low", "medium", "high", "critical"];

export function TaskFilters({
  filters,
  onChange,
  onReset,
}: {
  filters: TaskFilterState;
  onChange: (filters: TaskFilterState) => void;
  onReset: () => void;
}) {
  return (
    <section className="grid gap-3 rounded-lg border border-line bg-surface-raised p-4 md:grid-cols-[160px_160px_160px_1fr_auto]">
      <SelectFilter
        label="Status"
        onChange={(status) => onChange({ ...filters, status: status as TaskFilterState["status"] })}
        options={statusOptions}
        value={filters.status}
      />
      <SelectFilter
        label="Type"
        onChange={(type) => onChange({ ...filters, type: type as TaskFilterState["type"] })}
        options={typeOptions}
        value={filters.type}
      />
      <SelectFilter
        label="Priority"
        onChange={(priority) => onChange({ ...filters, priority: priority as TaskFilterState["priority"] })}
        options={priorityOptions}
        value={filters.priority}
      />
      <label className="space-y-1 text-sm font-medium text-ink-muted">
        <span>Tag</span>
        <input
          className="h-10 w-full rounded border border-line bg-field px-3 text-ink outline-none transition focus:border-action-border"
          onChange={(event) => onChange({ ...filters, tag: event.target.value })}
          type="search"
          value={filters.tag}
        />
      </label>
      <div className="flex items-end">
        <button
          className="h-10 rounded border border-line bg-surface px-3 text-sm font-medium text-ink-muted hover:bg-surface-muted hover:text-ink"
          onClick={onReset}
          type="button"
        >
          Reset
        </button>
      </div>
    </section>
  );
}

function SelectFilter({
  label,
  onChange,
  options,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  options: string[];
  value: string;
}) {
  return (
    <label className="space-y-1 text-sm font-medium text-ink-muted">
      <span>{label}</span>
      <select
        className="h-10 w-full rounded border border-line bg-field px-3 text-ink outline-none transition focus:border-action-border"
        onChange={(event) => onChange(event.target.value)}
        value={value}
      >
        {options.map((option) => (
          <option key={option} value={option}>{option === "all" ? "All" : option}</option>
        ))}
      </select>
    </label>
  );
}
