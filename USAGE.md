# heki - 使い方ガイド

## アプリケーション概要

**heki** は、Sony SensMe にインスパイアされた **ムード（気分）ベースのプレイリスト機能** を搭載したクロスプラットフォーム対応のデスクトップ音楽プレイヤーです。

Tauri 2 (Rust バックエンド) と React (TypeScript フロントエンド) で構築されており、以下の主な機能を備えています。

- **ライブラリ管理**: ローカルの音楽フォルダをスキャンし、メタデータ（タイトル、アーティスト、アルバム、ジャンル、アートワークなど）を自動的に読み取ります
- **ムード解析 (SensMe Channels)**: 楽曲のオーディオ波形を解析し、BPM・エネルギー・ヴァレンスから 8 つのムードカテゴリ（Energetic / Upbeat / Dance / Extreme / Emotional / Mellow / Relax / Lounge）に自動分類します
- **ムードマップ**: ムードカテゴリを可視化するインタラクティブなマップ画面から、気分に合った楽曲を一覧表示できます
- **プレイリスト**: 手動プレイリストの作成・編集に対応
- **再生機能**: シャッフル、リピート（Off / All / One）、キューの管理、再生回数カウント
- **アーティスト / アルバム / ジャンル ブラウズ**: カラムブラウザ形式でライブラリを絞り込み表示
- **検索**: タイトル・アーティスト・アルバムをまたいだインクリメンタル検索
- **ダークモード / ライトモード**: テーマ切り替えに対応
- **対応フォーマット**: MP3, FLAC, OGG, Opus, WAV, AAC, M4A, WMA, AIFF, APE

## アプリケーションの実行方法

### 前提条件

以下のツールが事前にインストールされている必要があります。

| ツール | バージョン目安 | 用途 |
|--------|---------------|------|
| [Node.js](https://nodejs.org/) | 18 以上 | フロントエンドビルド |
| [npm](https://www.npmjs.com/) | 9 以上 | パッケージ管理 |
| [Rust](https://www.rust-lang.org/tools/install) | 1.70 以上 | バックエンドビルド |
| [Tauri CLI](https://v2.tauri.app/start/prerequisites/) | 2.x | Tauri アプリのビルド・起動 |

プラットフォームごとの追加依存については [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/) を参照してください（Linux では `libwebkit2gtk-4.1-dev`, `libappindicator3-dev` 等が必要です）。

### セットアップ

```bash
# リポジトリのクローン
git clone <repository-url>
cd heki

# フロントエンドの依存関係をインストール
npm install
```

### 開発モードで起動

```bash
npm run tauri dev
```

Vite の開発サーバー（`http://localhost:1420`）が起動し、続いて Tauri のネイティブウィンドウが開きます。ホットリロードに対応しているため、フロントエンドのコード変更は即時反映されます。

### プロダクションビルド

```bash
npm run tauri build
```

各プラットフォーム向けのインストーラー / バイナリが `src-tauri/target/release/bundle/` 以下に生成されます。

### フロントエンドのみを確認する

```bash
# Vite 開発サーバーのみ起動（Tauri バックエンドなし）
npm run dev

# プロダクションビルド（フロントエンドのみ）
npm run build

# ビルド結果のプレビュー
npm run preview
```

> **注意**: フロントエンドのみの起動では Tauri API（ファイルスキャン、DB アクセス等）は利用できません。UI の確認用途としてご利用ください。

## 環境変数

| 環境変数 | 説明 | 値の例 |
|---------|------|-------|
| `TAURI_DEV_HOST` | 開発サーバーのホストアドレスを指定します。モバイルデバイスや他マシンからの接続テストに使用します | `192.168.1.100` |
| `RUST_LOG` | Rust バックエンドのログレベルを制御します（`env_logger` クレートが使用されます） | `debug`, `info`, `warn`, `error`, `heki_lib=debug` |
| `XDG_DATA_HOME` | (Linux) アプリケーションデータの保存先ベースディレクトリを指定します。未設定時は `$HOME/.local/share` がデフォルトです | `/home/user/.local/share` |
| `HOME` | (Linux / macOS) ホームディレクトリのパス。データ保存先の解決に使用されます | `/home/user` |
| `APPDATA` | (Windows) アプリケーションデータディレクトリ。データ保存先の解決に使用されます | `C:\Users\user\AppData\Roaming` |

### 使用例

```bash
# デバッグログを有効にして開発モードで起動
RUST_LOG=debug npm run tauri dev

# モバイルデバイスから接続できるようにホストを指定
TAURI_DEV_HOST=192.168.1.100 npm run tauri dev

# Linux でカスタムデータディレクトリを指定
XDG_DATA_HOME=/custom/data/path npm run tauri dev
```

## データの永続化

### データベース

アプリケーションのデータは **SQLite データベース** (`heki.db`) に保存されます。データベースファイルの保存先はプラットフォームにより異なります。

| OS | 保存先パス |
|----|-----------|
| Linux | `$XDG_DATA_HOME/heki/heki.db`（デフォルト: `~/.local/share/heki/heki.db`） |
| macOS | `~/Library/Application Support/heki/heki.db` |
| Windows | `%APPDATA%\heki\heki.db` |

ディレクトリが存在しない場合はアプリケーション起動時に自動作成されます。

### データベーススキーマ

データベースには以下の 3 つのテーブルが含まれます。

#### `tracks` テーブル
楽曲のメタデータおよび解析結果を保存します。

| カラム | 型 | 説明 |
|--------|-----|------|
| `id` | TEXT (PK) | UUID v4 による一意識別子 |
| `path` | TEXT (UNIQUE) | 楽曲ファイルの絶対パス |
| `title` | TEXT | タイトル |
| `artist` | TEXT | アーティスト名 |
| `album` | TEXT | アルバム名 |
| `album_artist` | TEXT | アルバムアーティスト名 |
| `genre` | TEXT | ジャンル |
| `track_number` | INTEGER | トラック番号 |
| `disc_number` | INTEGER | ディスク番号 |
| `year` | INTEGER | リリース年 |
| `duration_secs` | REAL | 再生時間（秒） |
| `file_format` | TEXT | ファイル形式（MP3, FLAC 等） |
| `bitrate` | INTEGER | ビットレート (kbps) |
| `sample_rate` | INTEGER | サンプルレート (Hz) |
| `bpm` | REAL | テンポ (BPM)（解析後に設定） |
| `energy` | REAL | エネルギー値 0.0〜1.0（解析後に設定） |
| `valence` | REAL | ヴァレンス値 0.0〜1.0（解析後に設定） |
| `mood` | TEXT | ムードカテゴリ（解析後に設定） |
| `play_count` | INTEGER | 再生回数 |
| `rating` | INTEGER | ユーザー評価 |
| `date_added` | TEXT | 追加日時 (RFC 3339) |
| `has_artwork` | INTEGER | アートワークの有無 (0/1) |

#### `playlists` テーブル
プレイリストの情報を保存します。

| カラム | 型 | 説明 |
|--------|-----|------|
| `id` | TEXT (PK) | UUID v4 による一意識別子 |
| `name` | TEXT | プレイリスト名 |
| `is_smart` | INTEGER | スマートプレイリストかどうか (0/1) |
| `mood_filter` | TEXT | ムードフィルター（スマートプレイリスト用） |
| `created_at` | TEXT | 作成日時 (RFC 3339) |

#### `playlist_tracks` テーブル
プレイリストと楽曲の関連付けを保存します。

| カラム | 型 | 説明 |
|--------|-----|------|
| `playlist_id` | TEXT (FK) | プレイリスト ID |
| `track_id` | TEXT (FK) | トラック ID |
| `position` | INTEGER | プレイリスト内の並び順 |

### フロントエンド設定

テーマ設定（ダークモード / ライトモード）はブラウザの **`localStorage`** にキー `heki-theme` として保存されます。値は `"dark"` または `"light"` です。

### データのバックアップ

データベースファイル (`heki.db`) をコピーするだけでライブラリ情報を完全にバックアップできます。復元時は同じ場所に配置してください。楽曲ファイル本体はデータベースには含まれず、ファイルパスのみが記録されています。

## Docker

heki は Docker によるコンテナ化に対応しています。`Dockerfile` はマルチステージビルドで構成されており、用途に応じてビルドターゲットを選択できます。

### ビルドステージ一覧

| ステージ名 | ベースイメージ | 用途 |
|---|---|---|
| `frontend-build` | `node:20-slim` | React フロントエンドのビルド（`npm run build` → `dist/`） |
| `tauri-build` | `rust:1.82-bookworm` | Tauri アプリケーションのフルビルド（deb / AppImage 生成） |
| `web` | `nginx:1.27-alpine` | ビルド済みフロントエンドを nginx で配信 |

### Web フロントエンドのイメージビルド

フロントエンドを nginx で配信する軽量イメージをビルドします。

```bash
docker build --target web -t heki-web .
```

### コンテナの起動

```bash
docker run -d -p 8080:80 heki-web
```

ブラウザで `http://localhost:8080` にアクセスすると Web UI が表示されます。

> **注意**: Web UI モードでは Tauri ネイティブ API（ファイルスキャン、オーディオ再生、DB アクセス等）は利用できません。UI の確認・デモ用途としてご利用ください。

### Tauri ネイティブビルド

Linux 向けのインストーラー（deb / AppImage）を Docker コンテナ内で生成します。

```bash
docker build --target tauri-build -t heki-tauri-build .
```

ビルド成果物は `tauri-build` ステージ内の `/app/src-tauri/target/release/bundle/` に生成されます。

## Docker Compose

`docker-compose.yaml` にはサービスとして以下が定義されています。

| サービス | ターゲット | ポート | プロファイル | 説明 |
|---|---|---|---|---|
| `web` | `web` | `8080:80` | (デフォルト) | nginx による Web フロントエンド配信 |
| `tauri-build` | `tauri-build` | — | `build` | Tauri ネイティブビルド（成果物をボリュームに出力） |

### Web サービスの起動

```bash
docker compose up web
```

`http://localhost:8080` でアクセスできます。バックグラウンド起動する場合は `-d` を付与してください。

### Tauri ビルドの実行

```bash
docker compose --profile build up tauri-build
```

ビルド成果物は `build-output` ボリュームに保存されます。

### 停止・クリーンアップ

```bash
# サービスの停止
docker compose down

# ボリュームも含めて削除
docker compose down -v
```

## CI/CD（GitHub Actions）

`.github/workflows/docker-build.yml` に以下の 2 つのジョブが定義されています。

### build-and-push

| 項目 | 内容 |
|---|---|
| トリガー | `main` ブランチへの push / pull_request |
| 処理内容 | `web` ターゲットの Docker イメージをビルドし GHCR へプッシュ |
| レジストリ | `ghcr.io/<owner>/heki` |
| タグ戦略 | ブランチ名、セマンティックバージョン（`v*` タグ）、コミット SHA |
| キャッシュ | GitHub Actions Cache (`type=gha`) |

pull_request 時はビルドのみ行い、プッシュは行いません。

### build-tauri

| 項目 | 内容 |
|---|---|
| トリガー | `v*` タグの push 時のみ |
| 処理内容 | Ubuntu 上で Tauri アプリをネイティブビルドし、GitHub Release にアップロード |
| 成果物 | `*.deb`、`*.AppImage` |

### タグによるリリースの流れ

```bash
git tag v0.1.0
git push origin v0.1.0
```

これにより以下が自動実行されます。

1. Docker イメージ `ghcr.io/<owner>/heki:0.1.0` がビルド・プッシュされる
2. Tauri ネイティブビルドが実行され、deb / AppImage が GitHub Release にアップロードされる

## Kubernetes

`k8s/` ディレクトリに Kubernetes マニフェストが用意されています。

### マニフェスト一覧

| ファイル | リソース | 説明 |
|---|---|---|
| `namespace.yaml` | Namespace | `heki` namespace の作成 |
| `deployment.yaml` | Deployment | Web フロントエンドの Pod を 2 レプリカでデプロイ |
| `service.yaml` | Service (ClusterIP) | Pod への内部ルーティング |
| `ingress.yaml` | Ingress | 外部からのアクセス（`heki.example.com`） |

### デプロイ手順

```bash
# 全マニフェストを一括適用
kubectl apply -f k8s/

# 状態確認
kubectl get all -n heki
```

### Ingress のホスト名変更

`k8s/ingress.yaml` の `spec.rules[].host` を実際のドメイン名に変更してください。

```yaml
rules:
  - host: heki.your-domain.com   # ← ここを変更
```

### Deployment のカスタマイズ

| パラメータ | 場所 | デフォルト |
|---|---|---|
| レプリカ数 | `deployment.yaml` → `spec.replicas` | `2` |
| イメージ | `deployment.yaml` → `spec.template.spec.containers[].image` | `ghcr.io/nord1441/heki:latest` |
| CPU リクエスト / リミット | `deployment.yaml` → `resources` | `50m` / `200m` |
| メモリ リクエスト / リミット | `deployment.yaml` → `resources` | `64Mi` / `128Mi` |

### クリーンアップ

```bash
kubectl delete -f k8s/
```

## Helm チャート

`helm/heki/` に Helm チャートが用意されており、パラメータ化されたデプロイが可能です。

### チャート構成

```
helm/heki/
├── Chart.yaml              # チャートメタデータ
├── values.yaml             # デフォルト値
└── templates/
    ├── _helpers.tpl         # テンプレートヘルパー関数
    ├── deployment.yaml      # Deployment
    ├── service.yaml         # Service
    ├── ingress.yaml         # Ingress（values で有効化）
    ├── hpa.yaml             # HorizontalPodAutoscaler（values で有効化）
    └── NOTES.txt            # インストール後のガイド表示
```

### インストール

```bash
# デフォルト設定でインストール
helm install heki helm/heki -n heki --create-namespace

# Ingress を有効にしてインストール
helm install heki helm/heki -n heki --create-namespace \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=heki.your-domain.com \
  --set ingress.hosts[0].paths[0].path=/ \
  --set ingress.hosts[0].paths[0].pathType=Prefix
```

### 主要パラメータ

| パラメータ | 説明 | デフォルト |
|---|---|---|
| `replicaCount` | Pod のレプリカ数 | `2` |
| `image.repository` | コンテナイメージのリポジトリ | `ghcr.io/nord1441/heki` |
| `image.tag` | イメージタグ（空の場合 `appVersion` を使用） | `""` |
| `image.pullPolicy` | イメージの pull ポリシー | `IfNotPresent` |
| `service.type` | Service の種別 | `ClusterIP` |
| `service.port` | Service のポート番号 | `80` |
| `ingress.enabled` | Ingress の有効化 | `false` |
| `ingress.className` | Ingress クラス名 | `nginx` |
| `ingress.hosts` | Ingress のホスト・パス設定 | `[{host: heki.example.com, ...}]` |
| `ingress.tls` | TLS 設定 | `[]` |
| `resources.requests.cpu` | CPU リクエスト | `50m` |
| `resources.requests.memory` | メモリリクエスト | `64Mi` |
| `resources.limits.cpu` | CPU リミット | `200m` |
| `resources.limits.memory` | メモリリミット | `128Mi` |
| `autoscaling.enabled` | HPA の有効化 | `false` |
| `autoscaling.minReplicas` | 最小レプリカ数 | `2` |
| `autoscaling.maxReplicas` | 最大レプリカ数 | `10` |
| `autoscaling.targetCPUUtilizationPercentage` | スケールアウトの CPU 閾値 | `80` |

### TLS 付き Ingress の例

```bash
helm install heki helm/heki -n heki --create-namespace \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=heki.your-domain.com \
  --set ingress.hosts[0].paths[0].path=/ \
  --set ingress.hosts[0].paths[0].pathType=Prefix \
  --set ingress.tls[0].secretName=heki-tls \
  --set ingress.tls[0].hosts[0]=heki.your-domain.com
```

### HPA（オートスケール）を有効にする

```bash
helm install heki helm/heki -n heki --create-namespace \
  --set autoscaling.enabled=true \
  --set autoscaling.minReplicas=2 \
  --set autoscaling.maxReplicas=10
```

### アップグレード

```bash
helm upgrade heki helm/heki -n heki --set image.tag=0.2.0
```

### アンインストール

```bash
helm uninstall heki -n heki
```
