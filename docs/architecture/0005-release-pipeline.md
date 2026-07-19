---
vibedoc: 1
id: ARCH-0005
kind: architecture
tags:
  - release
  - github-actions
  - distribution
related: []
---

# リリースパイプライン

vibe-docは、Gitタグを起点とするGitHub Actionsでプラットフォーム別バイナリを生成し、
GitHub Releaseへ公開する。

## バージョン管理

製品バージョンは、ルート`Cargo.toml`の`workspace.package.version`で一元管理する。
`vibe-doc`と`vibe-doc-core`は、このバージョンを継承する。

frontendは単独では公開せず、vibe-docバイナリへ埋め込む。そのため、
`frontend/package.json`のバージョンは`0.0.0`に固定する。

## リリース前の更新

リリースする変更を`main`へ反映する前に、ルート`Cargo.toml`のバージョンを更新する。
バージョン変更で`Cargo.lock`が更新された場合は、同じ変更に含める。

タグを作成する前に、次の検査が成功することを確認する。

```bash
pnpm --dir frontend install --frozen-lockfile
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --release --package vibe-doc
```

## タグの作成

`main`上のリリース対象コミットに、Cargoのバージョンと同じ`vX.Y.Z`形式のタグを付ける。
初回の`0.1.0`リリースでは、次を実行する。

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

タグと`workspace.package.version`が一致しない場合、Releaseワークフローは失敗する。

## GitHub Actions

通常のpushとPull Requestでは、`.github/workflows/ci.yml`がfmt、Clippy、テストを実行する。
CIとReleaseのRustジョブは、Cargoの依存関係とビルド成果物をキャッシュする。
検証ジョブはキャッシュを共有し、配布ビルドはターゲットごとにキャッシュを分離する。

`vX.Y.Z`タグをpushすると、`.github/workflows/release.yml`が次の処理を実行する。

1. タグ形式とCargoバージョンの一致を検証する。
2. fmt、Clippy、テストを実行する。
3. Linux x86_64、Windows x86_64、macOS x86_64、macOS aarch64向けにビルドする。
4. バイナリ、README、LICENSEをプラットフォーム別のアーカイブへまとめる。
5. 全アーカイブのSHA-256チェックサムを`SHA256SUMS`へ出力する。
6. GitHub Releaseを作成し、アーカイブとチェックサムを添付する。

リリースされたバイナリにはWeb UIが埋め込まれているため、利用環境にNode.js、pnpm、
`frontend/dist`は不要である。

## インストールスクリプト

リポジトリ直下の`install.sh`は、実行環境に対応する最新のUnix向けアーカイブと
`SHA256SUMS`をGitHub Releaseからダウンロードする。チェックサムを検証してから、
既定では`~/.local/bin/vibe-doc`へバイナリを配置する。

対象はLinux x86_64、macOS x86_64、macOS aarch64とする。`VIBE_DOC_VERSION`で
リリースバージョンを、`VIBE_DOC_INSTALL_DIR`で配置先を上書きできる。
