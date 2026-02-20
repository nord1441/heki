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
