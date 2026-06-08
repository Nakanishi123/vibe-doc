import type { TaskGroupsResponse, TaskSummary } from "./api-types";
import type { TaskFilterState } from "./app-types";

export function flattenTaskGroups(groups: TaskGroupsResponse) {
  return [...groups.active, ...groups.blocked, ...groups.done];
}

export function filterTasks(tasks: TaskSummary[], filters: TaskFilterState) {
  const tagNeedle = filters.tag.trim().toLowerCase();

  return tasks.filter((task) => {
    if (filters.status !== "all" && task.status !== filters.status) {
      return false;
    }
    if (filters.type !== "all" && task.type !== filters.type) {
      return false;
    }
    if (filters.priority !== "all" && task.priority !== filters.priority) {
      return false;
    }
    if (tagNeedle && !(task.tags ?? []).some((tag) => tag.toLowerCase().includes(tagNeedle))) {
      return false;
    }
    return true;
  });
}

export function groupFilteredTasks(tasks: TaskSummary[]) {
  return {
    active: tasks.filter((task) => task.status !== "blocked" && task.status !== "done" && task.status !== "dropped"),
    blocked: tasks.filter((task) => task.status === "blocked"),
    done: tasks.filter((task) => task.status === "done" || task.status === "dropped"),
  };
}
