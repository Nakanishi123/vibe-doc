import type { Mermaid, RenderResult } from "mermaid";
import type { ColorTheme } from "../components/ThemeContext";

let mermaidPromise: Promise<Mermaid> | undefined;
let renderQueue = Promise.resolve();

/**
 * AI生成のMermaidでよく使われるリテラルの `\n` を表示上の改行へ変換する。
 *
 * Mermaidでは図の種類によって `\n` の扱いが異なり、そのまま文字として表示される
 * 場合がある。Mermaidが各図で改行として扱える `<br/>` に描画直前で統一する。
 * `\\n` のようにバックスラッシュ自体がエスケープされている場合は変更しない。
 */
function normalizeMermaidLineBreaks(source: string): string {
  return source.replace(/(^|[^\\])\\n/g, "$1<br/>");
}

/**
 * バンドルされた固定バージョンのMermaidを必要になったときに一度だけ読み込む。
 *
 * 動的importによりMermaidを通常の画面から別チャンクへ分離する。失敗したPromiseも
 * 共有し、複数の図があるページで同じ読み込みを繰り返さない。
 */
export function loadMermaid(): Promise<Mermaid> {
  mermaidPromise ??= import("mermaid").then((module) => module.default);

  return mermaidPromise;
}

/**
 * アプリのカラーテーマをMermaidへ反映してSVGを生成する。
 *
 * Mermaidの設定はモジュール全体で共有されるため、初期化と描画を一組として直列化する。
 * これにより複数の図の描画中にテーマが切り替わっても、別の描画によるグローバル設定の
 * 上書きが混ざらない。エラー後も後続の図を描画できるようキュー自体は成功状態へ戻す。
 */
export function renderMermaid(
  mermaid: Mermaid,
  id: string,
  source: string,
  theme: ColorTheme,
): Promise<RenderResult> {
  const render = () => {
    const darkMode = theme === "dark";
    mermaid.initialize({
      securityLevel: "strict",
      startOnLoad: false,
      theme: darkMode ? "dark" : "default",
      themeVariables: { darkMode },
    });
    return mermaid.render(id, normalizeMermaidLineBreaks(source));
  };
  const result = renderQueue.then(render, render);
  renderQueue = result.then(
    () => undefined,
    () => undefined,
  );
  return result;
}
