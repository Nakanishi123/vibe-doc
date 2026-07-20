export type DocumentKind = "architecture" | "decision" | "task" | "research";

export interface DocumentSummary {
  id: string;
  title: string;
  path: string;
  kind?: DocumentKind;
  status?: string;
  tags: string[];
}

export interface DocumentDetail extends DocumentSummary {
  schemaVersion?: number;
  body: string;
  extra: Record<string, string>;
  related: DocumentSummary[];
  dependencies: DocumentSummary[];
  dependents: DocumentSummary[];
  backlinks: DocumentSummary[];
  bodyLinks: DocumentSummary[];
}

export interface TagSummary {
  name: string;
  count: number;
}

export interface TagDetail {
  name: string;
  documents: DocumentSummary[];
}

export interface LinkSummary {
  source: DocumentSummary;
  target: DocumentSummary;
  relation: "related" | "depends-on" | "markdown-link";
}

export interface LintItem {
  level: "error" | "warning";
  path: string;
  message: string;
}

export interface LintResponse {
  errors: number;
  warnings: number;
  diagnostics: LintItem[];
}
