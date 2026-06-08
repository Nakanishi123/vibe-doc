import type { DocumentKind, DocumentSummary, ValidationIssue, ValidationResponse } from "./api-types";
import type { FilterState, LoadState } from "./app-types";

export function filterDocuments(documents: DocumentSummary[], filters: FilterState) {
  const idNeedle = filters.id.trim();
  const titleNeedle = filters.title.trim().toLowerCase();
  const tagNeedle = filters.tag.trim().toLowerCase();

  return documents.filter((document) => {
    if (filters.kind !== "all" && document.kind !== filters.kind) {
      return false;
    }
    if (idNeedle && !String(document.id).includes(idNeedle)) {
      return false;
    }
    if (titleNeedle && !document.title.toLowerCase().includes(titleNeedle)) {
      return false;
    }
    if (tagNeedle && !(document.tags ?? []).some((tag) => tag.toLowerCase().includes(tagNeedle))) {
      return false;
    }
    return true;
  });
}

export function issuesForDocument(issues: ValidationIssue[], document: Pick<DocumentSummary, "id" | "path">) {
  return issues.filter((issue) => issue.document_id === document.id || issue.path === document.path);
}

export function countByKind(documents: DocumentSummary[]) {
  return documents.reduce(
    (counts, document) => ({ ...counts, [document.kind]: counts[document.kind] + 1 }),
    { spec: 0, design: 0, adr: 0, task: 0, "task-index": 0 } satisfies Record<DocumentKind, number>,
  );
}

export function validationLabel(validation: LoadState<ValidationResponse>) {
  if (validation.status === "loading") {
    return "validation: loading";
  }
  if (validation.status === "error") {
    return "validation: unavailable";
  }
  return `validation: ${validation.data.status}`;
}
