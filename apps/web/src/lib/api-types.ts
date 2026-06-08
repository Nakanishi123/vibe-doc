export type DocumentKind = "spec" | "design" | "adr" | "task" | "task-index";

export type TaskStatus = "planned" | "doing" | "blocked" | "done" | "dropped";

export type TaskType = "feature" | "bug" | "refactor" | "chore" | "docs" | "test" | "spike";

export type TaskPriority = "low" | "medium" | "high" | "critical";

export type AdrStatus =
  | "proposed"
  | "accepted"
  | "rejected"
  | "deprecated"
  | "superseded";

export type ValidationSeverity = "error" | "warning" | "info";

export type ValidationStatus = "ok" | "warning" | "error" | "unknown";

export interface ApiRoute {
  method: "GET" | "POST";
  path: string;
  description: string;
}

export interface HealthResponse {
  status: "ok";
  repository_root: string;
  document_count: number;
}

export interface DocumentSummary {
  id: number;
  title: string;
  kind: DocumentKind;
  path: string;
  tags?: string[];
}

export interface DocumentDetail extends DocumentSummary {
  frontmatter: Record<string, unknown>;
  markdown: string;
  html?: string;
  related_ids: RelatedDocumentId[];
  related_documents: RelatedDocument[];
  validation: ValidationIssue[];
}

export interface RelatedDocumentId {
  id: number;
  relation: "spec" | "design" | "adr" | "task" | "dependency";
}

export interface RelatedDocument {
  id: number;
  title: string;
  kind: DocumentKind;
  relation: "spec" | "design" | "adr" | "task" | "dependency";
}

export interface SpecSummary extends DocumentSummary {
  kind: "spec";
  related_designs: number[];
  related_tasks: number[];
}

export interface DesignSummary extends DocumentSummary {
  kind: "design";
  specs: number[];
  adrs: number[];
  related_tasks: number[];
}

export interface AdrSummary extends DocumentSummary {
  kind: "adr";
  status: AdrStatus;
  date?: string;
  related_designs: number[];
  supersedes: number[];
  superseded_by?: number;
}

export interface TaskSummary extends DocumentSummary {
  kind: "task";
  status: TaskStatus;
  type: TaskType;
  priority: TaskPriority;
  specs: number[];
  designs: number[];
  adrs: number[];
  depends_on: number[];
}

export interface TaskGroupsResponse {
  active: TaskSummary[];
  done: TaskSummary[];
  blocked: TaskSummary[];
}

export interface OverviewResponse {
  document_count: number;
  active_task_count: number;
  done_task_count: number;
  adr_count: number;
  validation: ValidationSummary;
  recently_updated: DocumentSummary[];
}

export interface ValidationSummary {
  status: ValidationStatus;
  error_count: number;
  warning_count: number;
}

export interface ValidationIssue {
  severity: ValidationSeverity;
  code: string;
  message: string;
  path?: string;
  document_id?: number;
  suggested_fix?: string;
}

export interface ValidationResponse extends ValidationSummary {
  incomplete: boolean;
  issues: ValidationIssue[];
}

export interface TaskContextResponse {
  task: TaskSummary;
  files: TaskContextFile[];
}

export interface TaskContextFile {
  path: string;
  role: "instructions" | "task" | "spec" | "design" | "adr";
  content: string;
}

export interface TaskMutationRequest {
  dry_run?: boolean;
  date?: string;
  result?: string;
}

export interface TaskLifecycleResponse {
  command: "start task" | "complete task";
  dry_run: boolean;
  task_id: number;
  changes: TaskLifecycleChange[];
}

export interface TaskLifecycleChange {
  path: string;
  action: "overwrite" | "create" | "delete" | "keep";
}

export interface RebuildIndexRequest {
  dry_run?: boolean;
}

export interface RebuildIndexResponse {
  command: "rebuild index";
  dry_run: boolean;
  path: string;
  action: "overwrite" | "keep";
  content: string;
}

export interface ApiError {
  code: string;
  message: string;
  path?: string;
  document_id?: number;
}
