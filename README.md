# vibe-doc

Git管理されたMarkdown文書を、ローカルで閲覧・検索・検査するためのツールです。

## Layout

- `crates/vibe-doc-core`: 文書モデル、解析、索引、リンク、lint、採番。
- `crates/vibe-doc`: CLI、JSON API、ローカルWebサーバー、UIアセット埋め込み。
- `frontend`: React、TypeScript、Viteによる読み取り専用のWeb UI。
- `docs`: vibe-docが扱うMarkdown文書。
- `tests/fixtures`: パーサーとlintの入力文書。
- `tests/integration`: CLI・APIの統合テスト。

## Development

```bash
pnpm --dir frontend install
cargo check
pnpm --dir frontend dev
```

フロントエンドの開発サーバーは`/api`を`127.0.0.1:3000`へプロキシします。

## Build and run

```bash
pnpm --dir frontend install --frozen-lockfile
cargo build --release
./target/release/vibe-doc serve
```

CargoビルドはViteを実行し、生成したWeb UIを`vibe-doc`へ埋め込みます。完成した
バイナリの実行時にはNode.js、pnpm、`frontend/dist`は不要です。
