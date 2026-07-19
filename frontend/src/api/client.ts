import type {
  DocumentDetail,
  DocumentSummary,
  LinkSummary,
  LintResponse,
  TagDetail,
  TagSummary,
} from "../types";

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(`/api${path}`, {
    ...init,
    headers: { Accept: "application/json" },
  });
  if (!response.ok) {
    throw new Error(response.status === 404 ? "Not found" : `Request failed (${response.status})`);
  }
  return response.json() as Promise<T>;
}

export const api = {
  reload() {
    return request<{ documentCount: number; diagnosticCount: number }>("/reload", {
      method: "POST",
    });
  },
  documents(params: Record<string, string> = {}) {
    const query = new URLSearchParams(
      Object.entries(params).filter(([, value]) => value.trim().length > 0),
    );
    const suffix = query.size > 0 ? `?${query}` : "";
    return request<DocumentSummary[]>(`/documents${suffix}`);
  },
  document(id: string) {
    return request<DocumentDetail>(`/documents/${encodeURIComponent(id)}`);
  },
  tags() {
    return request<TagSummary[]>("/tags");
  },
  tag(name: string) {
    return request<TagDetail>(`/tags/${encodeURIComponent(name)}`);
  },
  links() {
    return request<LinkSummary[]>("/links");
  },
  lint() {
    return request<LintResponse>("/lint");
  },
};
