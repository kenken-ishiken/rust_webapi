import ProductList from './components/ProductList';

export default function Home() {
  return (
    <div className="min-h-screen flex flex-col bg-white">
      {/* ヘッダー */}
      <header className="bg-white border-b w-full">
        <div className="container mx-auto px-4 py-3">
          <div className="flex justify-between items-center">
            <div className="flex items-center">
              <span className="text-red-600 font-bold text-2xl mr-1">Y!</span>
              <span className="text-xl font-semibold">ショッピング</span>
            </div>
            <div className="hidden md:flex space-x-4 text-sm">
              <a href="#" className="hover:text-red-600">ヘルプ</a>
              <a href="#" className="hover:text-red-600">お問い合わせ</a>
              <a href="#" className="hover:text-red-600">ログイン</a>
            </div>
          </div>
        </div>
      </header>

      {/* カテゴリナビゲーション */}
      <nav className="bg-gray-100 shadow-sm w-full">
        <div className="container mx-auto px-4">
          <div className="flex overflow-x-auto py-2 space-x-4 text-sm no-scrollbar">
            <a href="#" className="whitespace-nowrap hover:text-red-600">トップ</a>
            <a href="#" className="whitespace-nowrap hover:text-red-600">スポーツ</a>
            <a href="#" className="whitespace-nowrap hover:text-red-600">ファッション</a>
            <a href="#" className="whitespace-nowrap hover:text-red-600">家電</a>
            <a href="#" className="whitespace-nowrap hover:text-red-600">食品</a>
            <a href="#" className="whitespace-nowrap hover:text-red-600">美容・健康</a>
            <a href="#" className="whitespace-nowrap hover:text-red-600">インテリア</a>
            <a href="#" className="whitespace-nowrap hover:text-red-600">キッズ・ベビー</a>
            <a href="#" className="whitespace-nowrap hover:text-red-600">日用品</a>
            <a href="#" className="whitespace-nowrap hover:text-red-600">その他</a>
          </div>
        </div>
      </nav>

      {/* メインコンテンツ */}
      <main className="flex-grow bg-gray-50 py-6 w-full">
        <div className="container mx-auto px-4">
          <div className="mb-6">
            <h1 className="text-xl font-bold mb-2">スポーツシューズ</h1>
            <div className="flex flex-wrap text-xs text-gray-500">
              <a href="#" className="hover:underline mr-2">トップ</a>
              <span className="mr-2">&gt;</span>
              <a href="#" className="hover:underline mr-2">スポーツ・アウトドア</a>
              <span className="mr-2">&gt;</span>
              <a href="#" className="hover:underline mr-2">スポーツシューズ</a>
            </div>
          </div>
          <ProductList />
        </div>
      </main>

      {/* フッター */}
      <footer className="bg-white border-t py-8 w-full">
        <div className="container mx-auto px-4">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
            <div>
              <h3 className="font-bold text-sm mb-3">ショッピングガイド</h3>
              <ul className="text-xs space-y-2 text-gray-600">
                <li><a href="#" className="hover:underline">はじめての方へ</a></li>
                <li><a href="#" className="hover:underline">お支払い方法</a></li>
                <li><a href="#" className="hover:underline">配送・送料について</a></li>
                <li><a href="#" className="hover:underline">返品・交換について</a></li>
              </ul>
            </div>
            <div>
              <h3 className="font-bold text-sm mb-3">ユーザーサポート</h3>
              <ul className="text-xs space-y-2 text-gray-600">
                <li><a href="#" className="hover:underline">ヘルプ・お問い合わせ</a></li>
                <li><a href="#" className="hover:underline">よくある質問</a></li>
                <li><a href="#" className="hover:underline">保証について</a></li>
              </ul>
            </div>
            <div>
              <h3 className="font-bold text-sm mb-3">運営情報</h3>
              <ul className="text-xs space-y-2 text-gray-600">
                <li><a href="#" className="hover:underline">会社概要</a></li>
                <li><a href="#" className="hover:underline">採用情報</a></li>
                <li><a href="#" className="hover:underline">プレスリリース</a></li>
              </ul>
            </div>
            <div>
              <h3 className="font-bold text-sm mb-3">プライバシーと規約</h3>
              <ul className="text-xs space-y-2 text-gray-600">
                <li><a href="#" className="hover:underline">プライバシーポリシー</a></li>
                <li><a href="#" className="hover:underline">利用規約</a></li>
                <li><a href="#" className="hover:underline">特定商取引法に基づく表記</a></li>
              </ul>
            </div>
          </div>
          <div className="mt-8 pt-6 border-t text-center text-xs text-gray-500">
            <p> 2025 商品情報一覧アプリ - デモサイト（Yahoo! JAPANのクローンではありません）</p>
          </div>
        </div>
      </footer>
    </div>
  );
}
