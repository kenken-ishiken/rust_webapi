#!/bin/bash

# 要件定義Issue作成スクリプト
# Usage: ./scripts/create-requirement-issue.sh

set -e

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ヘルパー関数
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 必要なコマンドのチェック
check_dependencies() {
    print_info "依存関係をチェックしています..."
    
    if ! command -v gh &> /dev/null; then
        print_error "GitHub CLI (gh) がインストールされていません"
        print_info "インストール方法: https://cli.github.com/"
        exit 1
    fi
    
    if ! gh auth status &> /dev/null; then
        print_error "GitHub CLI にログインしていません"
        print_info "ログイン方法: gh auth login"
        exit 1
    fi
    
    print_success "依存関係の確認完了"
}

# Issueタイトルの入力
get_issue_title() {
    echo
    print_info "要件定義のタイトルを入力してください（例: ユーザー認証API）:"
    read -r title
    
    if [[ -z "$title" ]]; then
        print_error "タイトルは必須です"
        get_issue_title
        return
    fi
    
    ISSUE_TITLE="[REQ] $title"
    print_success "タイトル: $ISSUE_TITLE"
}

# コンポーネントの選択
select_component() {
    echo
    print_info "関連するコンポーネントを選択してください:"
    echo "1) API"
    echo "2) Database"
    echo "3) Auth"
    echo "4) Frontend"
    echo "5) その他"
    
    read -r -p "選択 (1-5): " component_choice
    
    case $component_choice in
        1) COMPONENT_LABEL="component/api" ;;
        2) COMPONENT_LABEL="component/database" ;;
        3) COMPONENT_LABEL="component/auth" ;;
        4) COMPONENT_LABEL="component/frontend" ;;
        5) COMPONENT_LABEL="" ;;
        *) 
            print_warning "無効な選択です。再度選択してください。"
            select_component
            return
            ;;
    esac
    
    if [[ -n "$COMPONENT_LABEL" ]]; then
        print_success "コンポーネント: $COMPONENT_LABEL"
    fi
}

# 優先度の選択
select_priority() {
    echo
    print_info "優先度を選択してください:"
    echo "1) Critical (クリティカル)"
    echo "2) High (高優先度)"
    echo "3) Medium (中優先度)"
    echo "4) Low (低優先度)"
    
    read -r -p "選択 (1-4): " priority_choice
    
    case $priority_choice in
        1) PRIORITY_LABEL="priority/critical" ;;
        2) PRIORITY_LABEL="priority/high" ;;
        3) PRIORITY_LABEL="priority/medium" ;;
        4) PRIORITY_LABEL="priority/low" ;;
        *) 
            print_warning "無効な選択です。再度選択してください。"
            select_priority
            return
            ;;
    esac
    
    print_success "優先度: $PRIORITY_LABEL"
}

# 機能タイプの選択
select_feature_type() {
    echo
    print_info "機能タイプを選択してください:"
    echo "1) Feature (新機能)"
    echo "2) Enhancement (機能拡張)"
    echo "3) Bugfix (バグ修正)"
    echo "4) Refactor (リファクタリング)"
    
    read -r -p "選択 (1-4): " type_choice
    
    case $type_choice in
        1) TYPE_LABEL="type/feature" ;;
        2) TYPE_LABEL="type/enhancement" ;;
        3) TYPE_LABEL="type/bugfix" ;;
        4) TYPE_LABEL="type/refactor" ;;
        *) 
            print_warning "無効な選択です。再度選択してください。"
            select_feature_type
            return
            ;;
    esac
    
    print_success "機能タイプ: $TYPE_LABEL"
}

# 担当者の選択
select_assignee() {
    echo
    print_info "担当者を指定しますか？ (y/n) [デフォルト: 自分]:"
    read -r assign_choice
    
    case $assign_choice in
        [Yy]* )
            print_info "担当者のGitHubユーザー名を入力してください:"
            read -r assignee
            if [[ -n "$assignee" ]]; then
                ASSIGNEE="--assignee $assignee"
                print_success "担当者: $assignee"
            fi
            ;;
        * )
            ASSIGNEE="--assignee @me"
            print_success "担当者: 自分"
            ;;
    esac
}

# ラベルの構築
build_labels() {
    LABELS="requirement,status/draft"
    
    if [[ -n "$COMPONENT_LABEL" ]]; then
        LABELS="$LABELS,$COMPONENT_LABEL"
    fi
    
    if [[ -n "$PRIORITY_LABEL" ]]; then
        LABELS="$LABELS,$PRIORITY_LABEL"
    fi
    
    if [[ -n "$TYPE_LABEL" ]]; then
        LABELS="$LABELS,$TYPE_LABEL"
    fi
    
    print_success "ラベル: $LABELS"
}

# Issue作成の確認
confirm_creation() {
    echo
    print_info "=== Issue作成内容の確認 ==="
    echo "タイトル: $ISSUE_TITLE"
    echo "ラベル: $LABELS"
    echo "担当者: $ASSIGNEE"
    echo "テンプレート: requirement-definition.md"
    echo
    
    print_warning "この内容でIssueを作成しますか？ (y/n):"
    read -r confirm
    
    case $confirm in
        [Yy]* ) return 0 ;;
        * ) 
            print_info "Issue作成をキャンセルしました"
            exit 0
            ;;
    esac
}

# Issue作成
create_issue() {
    print_info "Issueを作成しています..."
    
    # GitHub CLI コマンドの実行
    if ISSUE_URL=$(gh issue create \
        --title "$ISSUE_TITLE" \
        --template requirement-definition.md \
        --label "$LABELS" \
        $ASSIGNEE 2>&1); then
        
        print_success "Issueが正常に作成されました！"
        print_info "Issue URL: $ISSUE_URL"
        
        # ブラウザで開くかどうか確認
        echo
        print_info "ブラウザでIssueを開きますか？ (y/n):"
        read -r open_browser
        
        case $open_browser in
            [Yy]* )
                gh issue view --web "${ISSUE_URL##*/}"
                ;;
        esac
        
    else
        print_error "Issue作成に失敗しました: $ISSUE_URL"
        exit 1
    fi
}

# 後続作業の案内
show_next_steps() {
    echo
    print_info "=== 次のステップ ==="
    echo "1. 作成されたIssueに詳細な要件を記入してください"
    echo "2. ステークホルダーにレビューを依頼してください"
    echo "3. 承認後、ラベルを 'status/review' に変更してください"
    echo
    print_info "ラベル変更コマンド例:"
    echo "gh issue edit ISSUE_NUMBER --remove-label status/draft --add-label status/review"
    echo
    print_info "要件定義ワークフローの詳細: docs/requirement-workflow.yml"
}

# メイン処理
main() {
    echo
    print_info "🎯 要件定義Issue作成ツール"
    echo "=============================="
    
    check_dependencies
    get_issue_title
    select_component
    select_priority
    select_feature_type
    select_assignee
    build_labels
    confirm_creation
    create_issue
    show_next_steps
    
    print_success "要件定義Issue作成が完了しました！"
}

# スクリプト実行
main "$@"
