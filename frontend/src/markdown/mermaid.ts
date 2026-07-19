const MERMAID_MODULE_URL = "https://cdn.jsdelivr.net/npm/mermaid@11.12.0/dist/mermaid.esm.min.mjs";

export interface MermaidRenderResult {
  svg: string;
  bindFunctions?: (element: Element) => void;
}

interface MermaidApi {
  initialize(configuration: { securityLevel: "strict"; startOnLoad: boolean }): void;
  render(id: string, source: string): Promise<MermaidRenderResult>;
}

interface MermaidModule {
  default: MermaidApi;
}

let mermaidPromise: Promise<MermaidApi> | undefined;

/**
 * 固定バージョンのMermaidをCDNから一度だけ読み込み、安全な設定で初期化する。
 *
 * URLを変数経由の動的importにすることでMermaidをViteの成果物へ含めない。失敗した
 * Promiseも共有し、複数の図があるページでCDN障害時に同じ要求を繰り返さない。
 */
export function loadMermaid(): Promise<MermaidApi> {
  mermaidPromise ??= import(/* @vite-ignore */ MERMAID_MODULE_URL).then((module) => {
    const mermaid = (module as MermaidModule).default;
    mermaid.initialize({ securityLevel: "strict", startOnLoad: false });
    return mermaid;
  });

  return mermaidPromise;
}
