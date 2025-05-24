# Scripts

このディレクトリには、開発とCI/CDで使用される補助スクリプトが含まれています。

## 利用可能なスクリプト

### coverage.sh

テストカバレッジレポートを生成するスクリプトです。

**前提条件:**
- `llvm-tools-preview` コンポーネントがインストールされている必要があります

```bash
# 前提条件のインストール
rustup component add llvm-tools-preview
```

**使用方法:**

```bash
# カバレッジレポートを生成
./scripts/coverage.sh
```

**出力:**
- `target/coverage/` ディレクトリにHTMLレポートが生成されます
- `lcov.info` ファイルが生成されます（CI用）
- `filtered_lcov.info` ファイルが生成されます（テストファイルを除外した版）

**生成されるファイル:**
- `target/coverage/index.html` - メインのカバレッジレポート
- `lcov.info` - LCOV形式のカバレッジデータ
- `filtered_lcov.info` - テストファイルを除外したカバレッジデータ

**例:**

```bash
# カバレッジを生成してブラウザで確認
./scripts/coverage.sh
open target/coverage/index.html
```

## 開発時の使用パターン

### CI/CDでの使用

GitHub Actionsやその他のCI/CDシステムでカバレッジレポートを生成する場合：

```yaml
- name: Generate coverage report
  run: ./scripts/coverage.sh

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    file: ./filtered_lcov.info
```

### ローカル開発

開発中にカバレッジを確認したい場合：

```bash
# テスト実行 → カバレッジ生成 → ブラウザで確認
cargo test && ./scripts/coverage.sh && open target/coverage/index.html
```

## スクリプトの追加

新しいスクリプトを追加する場合は、以下のガイドラインに従ってください：

1. **実行可能権限**: `chmod +x scripts/new_script.sh`
2. **シバン行**: `#!/bin/bash` を先頭に追加
3. **エラーハンドリング**: `set -euo pipefail` を追加
4. **ドキュメント**: このREADMEに使用方法を追加
5. **テスト**: スクリプトが正常に動作することを確認

### スクリプトテンプレート

```bash
#!/bin/bash
set -euo pipefail

# Script description
# Usage: ./scripts/new_script.sh [options]

# Script implementation here
echo "Hello, World!"
```