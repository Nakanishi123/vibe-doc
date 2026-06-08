import { useMemo, useState } from "react";

import { IdChips, TagList } from "../components/document-badges";
import { TaskFilters } from "../components/task-filters";
import { LoadBoundary, Panel, ScreenHeading } from "../components/chrome";
import type { TaskGroupsResponse, TaskSummary, ValidationIssue } from "../lib/api-types";
import type { LoadState, Navigate } from "../lib/app-types";
import { initialTaskFilters } from "../lib/app-types";
import { issuesForDocument } from "../lib/documents";
import { filterTasks, flattenTaskGroups, groupFilteredTasks } from "../lib/tasks";

export function TasksScreen({
  navigate,
  tasks,
  validationIssues,
}: {
  navigate: Navigate;
  tasks: LoadState<TaskGroupsResponse>;
  validationIssues: ValidationIssue[];
}) {
  const [filters, setFilters] = useState(initialTaskFilters);
  const allTasks = useMemo(() => (tasks.status === "ready" ? flattenTaskGroups(tasks.data) : []), [tasks]);
  const filtered = useMemo(() => filterTasks(allTasks, filters), [allTasks, filters]);
  const groups = useMemo(() => groupFilteredTasks(filtered), [filtered]);

  return (
    <div className="space-y-5">
      <ScreenHeading
        eyebrow="Tasks"
        title="Implementation task board"
        meta={tasks.status === "ready" ? `${filtered.length} of ${allTasks.length}` : "Loading"}
      />
      <TaskFilters filters={filters} onChange={setFilters} onReset={() => setFilters(initialTaskFilters)} />
      <LoadBoundary state={tasks}>
        <div className="grid gap-5">
          <TaskGroup
            emptyMessage="No active tasks match these filters."
            navigate={navigate}
            tasks={groups.active}
            title="Active"
            validationIssues={validationIssues}
          />
          <TaskGroup
            emptyMessage="No blocked tasks match these filters."
            navigate={navigate}
            tasks={groups.blocked}
            title="Blocked"
            validationIssues={validationIssues}
          />
          <TaskGroup
            emptyMessage="No done or dropped tasks match these filters."
            navigate={navigate}
            tasks={groups.done}
            title="Done"
            validationIssues={validationIssues}
          />
        </div>
      </LoadBoundary>
    </div>
  );
}

function TaskGroup({
  emptyMessage,
  navigate,
  tasks,
  title,
  validationIssues,
}: {
  emptyMessage: string;
  navigate: Navigate;
  tasks: TaskSummary[];
  title: string;
  validationIssues: ValidationIssue[];
}) {
  return (
    <Panel title={`${title} tasks (${tasks.length})`}>
      <div className="overflow-hidden rounded border border-slate-200">
        <table className="w-full table-fixed text-left text-sm">
          <thead className="bg-surface-muted text-xs uppercase text-ink-soft">
            <tr>
              <th className="w-20 px-4 py-3">ID</th>
              <th className="px-4 py-3">Task</th>
              <th className="hidden w-28 px-4 py-3 md:table-cell">Status</th>
              <th className="hidden w-28 px-4 py-3 md:table-cell">Type</th>
              <th className="hidden w-28 px-4 py-3 lg:table-cell">Priority</th>
              <th className="hidden w-36 px-4 py-3 xl:table-cell">Depends</th>
              <th className="w-20 px-4 py-3">Issues</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-200 bg-surface-raised">
            {tasks.length === 0 ? (
              <tr>
                <td className="px-4 py-8 text-center text-ink-muted" colSpan={7}>{emptyMessage}</td>
              </tr>
            ) : (
              tasks.map((task) => (
                <tr className="align-top hover:bg-surface-muted" key={task.id}>
                  <td className="px-4 py-3 font-medium" translate="no">#{task.id}</td>
                  <td className="min-w-0 px-4 py-3">
                    <button
                      className="block max-w-full truncate text-left font-medium text-action hover:underline"
                      onClick={() => navigate({ name: "task-detail", id: task.id }, `/tasks/${task.id}`)}
                      type="button"
                    >
                      {task.title}
                    </button>
                    <div className="mt-2 flex flex-wrap gap-2">
                      <TagList tags={task.tags ?? []} />
                    </div>
                    <div className="mt-2 grid gap-2 text-xs text-ink-muted sm:grid-cols-3 md:hidden" translate="no">
                      <span>{task.status}</span>
                      <span>{task.type}</span>
                      <span>{task.priority}</span>
                    </div>
                  </td>
                  <td className="hidden px-4 py-3 md:table-cell" translate="no"><StatusBadge value={task.status} /></td>
                  <td className="hidden px-4 py-3 md:table-cell" translate="no">{task.type}</td>
                  <td className="hidden px-4 py-3 lg:table-cell" translate="no">{task.priority}</td>
                  <td className="hidden px-4 py-3 xl:table-cell"><IdChips ids={task.depends_on} navigate={navigate} /></td>
                  <td className="px-4 py-3" translate="no">{issuesForDocument(validationIssues, task).length}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </Panel>
  );
}

function StatusBadge({ value }: { value: string }) {
  const className = value === "blocked"
    ? "border-amber-200 bg-attention-soft text-attention"
    : value === "done" || value === "dropped"
      ? "border-slate-200 bg-surface-muted text-ink-muted"
      : "border-action-border bg-action-soft text-action";

  return <span className={`rounded border px-2 py-1 text-xs font-medium ${className}`}>{value}</span>;
}
