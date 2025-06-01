#!/bin/bash

# 要件定義Issue管理ユーティリティ
# Usage: ./scripts/manage-requirements.sh [command] [options]

set -e

# カラー定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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

print_header() {
    echo -e "${PURPLE}🎯 $1${NC}"
    echo "=================================="
}

# 使用方法の表示
show_usage() {
    echo "要件定義Issue管理ユーティリティ"
    echo
    echo "使用方法:"
    echo "  $0 list [status]           - 要件定義Issueを一覧表示"
    echo "  $0 status <issue_number>   - Issue状態を更新"
    echo "  $0 review <issue_number>   - レビュー準備（draft → review）"
    echo "  $0 approve <issue_number>  - 承認（review → approved）"
    echo "  $0 start <issue_number>    - 実装開始（approved → in-progress）"
    echo "  $0 testing <issue_number>  - テスト開始（in-progress → testing）"
    echo "  $0 done <issue_number>     - 完了（testing → done）"
    echo "  $0 dashboard               - 要件定義ダッシュボード表示"
    echo "  $0 metrics                 - メトリクス表示"
    echo
    echo "例:"
    echo "  $0 list                    # すべての要件定義Issueを表示"
    echo "  $0 list draft              # ドラフト状態のIssueを表示"
    echo "  $0 review 123              # Issue #123をレビュー状態に変更"
    echo "  $0 dashboard               # ダッシュボード表示"
}

# 必要なコマンドのチェック
check_dependencies() {
    if ! command -v gh &> /dev/null; then
        print_error "GitHub CLI (gh) がインストールされていません"
        exit 1
    fi
    
    if ! gh auth status &> /dev/null; then
        print_error "GitHub CLI にログインしていません"
        exit 1
    fi
}

# 要件定義Issueの一覧表示
list_requirements() {
    local status_filter="$1"
    
    print_header "要件定義Issue一覧"
    
    if [[ -n "$status_filter" ]]; then
        print_info "フィルター: status/$status_filter"
        gh issue list --label "requirement,status/$status_filter" --state all \
            --json number,title,state,labels,assignees,createdAt,updatedAt \
            --template '{{range .}}{{printf "#%-4d" .number}} {{.state | printf "%-6s"}} {{.title | printf "%-50s"}} {{range .assignees}}@{{.login}} {{end}}{{printf "\n"}}{{end}}'
    else
        gh issue list --label requirement --state all \
            --json number,title,state,labels,assignees,createdAt,updatedAt \
            --template '{{range .}}{{printf "#%-4d" .number}} {{.state | printf "%-6s"}} {{.title | printf "%-50s"}} {{range .assignees}}@{{.login}} {{end}}{{printf "\n"}}{{end}}'
    fi
}

# Issue状態の更新
update_issue_status() {
    local issue_number="$1"
    local old_status="$2"
    local new_status="$3"
    local close_issue="$4"
    
    print_info "Issue #$issue_number の状態を更新しています: $old_status → $new_status"
    
    local cmd="gh issue edit $issue_number --remove-label status/$old_status --add-label status/$new_status"
    
    if [[ "$close_issue" == "true" ]]; then
        cmd="$cmd --state closed"
    fi
    
    if eval "$cmd"; then
        print_success "Issue #$issue_number の状態を更新しました"
        
        # 自動コメント追加
        local comment=""
        case "$new_status" in
            "review")
                comment="🔍 レビュー準備完了。ステークホルダーの皆様、レビューをお願いします。"
                ;;
            "approved")
                comment="✅ 要件定義が承認されました。実装フェーズに移行できます。"
                ;;
            "in-progress")
                comment="⚙️ 実装を開始しました。TDDサイクルに従って開発を進めます。"
                ;;
            "testing")
                comment="🧪 実装完了。テスト・検証フェーズを開始します。"
                ;;
            "done")
                comment="🎉 要件定義が完了しました。本番環境にリリース済みです。"
                ;;
        esac
        
        if [[ -n "$comment" ]]; then
            gh issue comment "$issue_number" --body "$comment"
        fi
    else
        print_error "Issue #$issue_number の状態更新に失敗しました"
        return 1
    fi
}

# レビュー準備
prepare_review() {
    local issue_number="$1"
    update_issue_status "$issue_number" "draft" "review" "false"
}

# 承認
approve_requirement() {
    local issue_number="$1"
    update_issue_status "$issue_number" "review" "approved" "false"
}

# 実装開始
start_implementation() {
    local issue_number="$1"
    update_issue_status "$issue_number" "approved" "in-progress" "false"
    
    # 実装ブランチの作成提案
    local issue_title=$(gh issue view "$issue_number" --json title --jq '.title')
    local branch_name=$(echo "$issue_title" | sed 's/\[REQ\] //g' | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | tr -cd '[:alnum:]-')
    
    print_info "実装ブランチを作成することをお勧めします:"
    print_info "git checkout -b feature/req-$issue_number-$branch_name"
}

# テスト開始
start_testing() {
    local issue_number="$1"
    update_issue_status "$issue_number" "in-progress" "testing" "false"
}

# 完了
complete_requirement() {
    local issue_number="$1"
    update_issue_status "$issue_number" "testing" "done" "true"
}

# ダッシュボード表示
show_dashboard() {
    print_header "要件定義ダッシュボード"
    
    # 状態別の統計
    echo -e "${CYAN}📊 状態別統計${NC}"
    echo "===================="
    
    local draft_count=$(gh issue list --label "requirement,status/draft" --state all --json number | jq length)
    local review_count=$(gh issue list --label "requirement,status/review" --state all --json number | jq length)
    local approved_count=$(gh issue list --label "requirement,status/approved" --state all --json number | jq length)
    local progress_count=$(gh issue list --label "requirement,status/in-progress" --state all --json number | jq length)
    local testing_count=$(gh issue list --label "requirement,status/testing" --state all --json number | jq length)
    local done_count=$(gh issue list --label "requirement,status/done" --state all --json number | jq length)
    
    echo "🗂️  ドラフト     : $draft_count 件"
    echo "🔍 レビュー中   : $review_count 件"
    echo "✅ 承認済み     : $approved_count 件"
    echo "⚙️  実装中       : $progress_count 件"
    echo "🧪 テスト中     : $testing_count 件"
    echo "🎉 完了         : $done_count 件"
    
    echo
    echo -e "${CYAN}🚨 注意が必要な項目${NC}"
    echo "===================="
    
    # 長期間レビュー待ちの項目
    echo "📅 長期間レビュー待ち（7日以上）:"
    gh issue list --label "requirement,status/review" --state open \
        --json number,title,createdAt \
        --jq '.[] | select(now - (.createdAt | fromdateiso8601) > 604800) | "#\(.number) \(.title)"' || echo "該当なし"
    
    echo
    echo "📅 長期間実装中（14日以上）:"
    gh issue list --label "requirement,status/in-progress" --state open \
        --json number,title,createdAt \
        --jq '.[] | select(now - (.createdAt | fromdateiso8601) > 1209600) | "#\(.number) \(.title)"' || echo "該当なし"
    
    echo
    echo -e "${CYAN}📈 最近の活動${NC}"
    echo "=================="
    echo "最近更新された要件定義（上位5件）:"
    gh issue list --label requirement --state all --limit 5 \
        --json number,title,updatedAt \
        --template '{{range .}}#{{.number}} {{.title}} ({{timeago .updatedAt}}){{"\n"}}{{end}}'
}

# メトリクス表示
show_metrics() {
    print_header "要件定義メトリクス"
    
    echo -e "${CYAN}📊 完了率${NC}"
    echo "=============="
    
    local total_count=$(gh issue list --label requirement --state all --json number | jq length)
    local done_count=$(gh issue list --label "requirement,status/done" --state all --json number | jq length)
    
    if [[ $total_count -gt 0 ]]; then
        local completion_rate=$((done_count * 100 / total_count))
        echo "総要件数: $total_count"
        echo "完了数: $done_count"
        echo "完了率: $completion_rate%"
    else
        echo "要件定義がありません"
    fi
    
    echo
    echo -e "${CYAN}⏱️  平均処理時間${NC}"
    echo "=================="
    echo "（完了した要件定義の平均処理時間）"
    
    # 完了した要件の作成日と完了日を取得して平均を計算
    # 注意: これは簡易版の実装です。より正確な計算には追加のロジックが必要です
    gh issue list --label "requirement,status/done" --state closed --limit 10 \
        --json number,title,createdAt,closedAt \
        --template '{{range .}}#{{.number}} {{.title}}: {{timeago .createdAt}} - {{timeago .closedAt}}{{"\n"}}{{end}}'
    
    echo
    echo -e "${CYAN}🏷️  コンポーネント別分布${NC}"
    echo "======================"
    
    local api_count=$(gh issue list --label "requirement,component/api" --state all --json number | jq length)
    local db_count=$(gh issue list --label "requirement,component/database" --state all --json number | jq length)
    local auth_count=$(gh issue list --label "requirement,component/auth" --state all --json number | jq length)
    local frontend_count=$(gh issue list --label "requirement,component/frontend" --state all --json number | jq length)
    
    echo "API関連: $api_count 件"
    echo "データベース関連: $db_count 件"
    echo "認証関連: $auth_count 件"
    echo "フロントエンド関連: $frontend_count 件"
}

# メイン処理
main() {
    check_dependencies
    
    case "${1:-}" in
        "list")
            list_requirements "$2"
            ;;
        "status")
            if [[ -z "$2" ]]; then
                print_error "Issue番号を指定してください"
                show_usage
                exit 1
            fi
            # 手動での状態変更（対話式）
            print_info "Issue #$2 の状態を変更します"
            print_info "現在の状態を確認しています..."
            gh issue view "$2" --json labels --jq '.labels[].name' | grep "status/"
            ;;
        "review")
            if [[ -z "$2" ]]; then
                print_error "Issue番号を指定してください"
                exit 1
            fi
            prepare_review "$2"
            ;;
        "approve")
            if [[ -z "$2" ]]; then
                print_error "Issue番号を指定してください"
                exit 1
            fi
            approve_requirement "$2"
            ;;
        "start")
            if [[ -z "$2" ]]; then
                print_error "Issue番号を指定してください"
                exit 1
            fi
            start_implementation "$2"
            ;;
        "testing")
            if [[ -z "$2" ]]; then
                print_error "Issue番号を指定してください"
                exit 1
            fi
            start_testing "$2"
            ;;
        "done")
            if [[ -z "$2" ]]; then
                print_error "Issue番号を指定してください"
                exit 1
            fi
            complete_requirement "$2"
            ;;
        "dashboard")
            show_dashboard
            ;;
        "metrics")
            show_metrics
            ;;
        "help"|"-h"|"--help")
            show_usage
            ;;
        "")
            show_usage
            ;;
        *)
            print_error "不明なコマンド: $1"
            show_usage
            exit 1
            ;;
    esac
}

# スクリプト実行
main "$@"
