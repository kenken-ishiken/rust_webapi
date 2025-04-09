'use client';

import { useState, useEffect } from 'react';
import Image from 'next/image';

interface Item {
  id: number;
  name: string;
  description: string | null;
  price: number;
  imageUrl: string;
  category: string;
  brand: string;
  rating: number;
}

// モックデータを拡張
const mockItems: Item[] = [
  {
    id: 1,
    name: "ナイキ エアマックス 90",
    description: "クラシックなデザインと優れた履き心地を実現したランニングシューズ。",
    price: 12000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "NIKE",
    rating: 4.5
  },
  {
    id: 2,
    name: "アディダス ウルトラブースト",
    description: "反発性に優れたクッショニングを備えた高性能ランニングシューズ。",
    price: 15000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "adidas",
    rating: 4.7
  },
  {
    id: 3,
    name: "プーマ RS-X",
    description: "90年代のランニングテクノロジーにインスパイアされたスタイリッシュなスニーカー。",
    price: 10000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "PUMA",
    rating: 4.2
  },
  {
    id: 4,
    name: "アシックス ゲルカヤノ 28",
    description: "長距離ランナー向けの安定性と快適さを両立したプレミアムランニングシューズ。",
    price: 14000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "ASICS",
    rating: 4.6
  },
  {
    id: 5,
    name: "ニューバランス 990v5",
    description: "クラシックなデザインと現代のテクノロジーを組み合わせた高品質なランニングシューズ。",
    price: 18000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "New Balance",
    rating: 4.8
  },
  {
    id: 6,
    name: "ミズノ ウェーブライダー 25",
    description: "軽量で反発性に優れた日本製の高性能ランニングシューズ。",
    price: 13000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "MIZUNO",
    rating: 4.4
  },
  {
    id: 7,
    name: "リーボック クラシックレザー",
    description: "伝統的なデザインと快適な履き心地が特徴のレトロスニーカー。",
    price: 9000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "Reebok",
    rating: 4.3
  },
  {
    id: 8,
    name: "サロモン スピードクロス 5",
    description: "トレイルランニング向けの優れたグリップ力と耐久性を持つアウトドアシューズ。",
    price: 16000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "SALOMON",
    rating: 4.6
  },
  {
    id: 9,
    name: "オン クラウドフロー",
    description: "スイス発のブランドによる革新的なクッショニングシステムを搭載したランニングシューズ。",
    price: 15000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "On",
    rating: 4.7
  },
  {
    id: 10,
    name: "ホカ クリフトン 8",
    description: "軽量でクッション性に優れた人気のロードランニングシューズ。",
    price: 14000,
    imageUrl: "https://placehold.jp/300x300.png",
    category: "シューズ",
    brand: "HOKA",
    rating: 4.5
  }
];

// カテゴリと価格帯のフィルターオプション
const categories = ["すべて", "シューズ", "ウェア", "アクセサリー"];
const priceRanges = [
  { label: "すべて", min: 0, max: Infinity },
  { label: "〜¥10,000", min: 0, max: 10000 },
  { label: "¥10,000〜¥15,000", min: 10000, max: 15000 },
  { label: "¥15,000〜", min: 15000, max: Infinity }
];
const sortOptions = [
  { label: "おすすめ順", value: "recommended" },
  { label: "価格が安い順", value: "priceAsc" },
  { label: "価格が高い順", value: "priceDesc" },
  { label: "評価が高い順", value: "ratingDesc" }
];

export default function ProductList() {
  const [items, setItems] = useState<Item[]>(mockItems);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState("すべて");
  const [selectedPriceRange, setSelectedPriceRange] = useState(priceRanges[0]);
  const [sortBy, setSortBy] = useState(sortOptions[0].value);
  const [currentPage, setCurrentPage] = useState(1);
  const itemsPerPage = 4;

  // 検索とフィルタリングの処理
  useEffect(() => {
    setLoading(true);
    
    try {
      // 検索クエリ、カテゴリ、価格範囲でフィルタリング
      let filteredItems = mockItems.filter(item => {
        const matchesSearch = searchQuery === "" || 
          item.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
          (item.description && item.description.toLowerCase().includes(searchQuery.toLowerCase()));
        
        const matchesCategory = selectedCategory === "すべて" || item.category === selectedCategory;
        
        const matchesPriceRange = item.price >= selectedPriceRange.min && item.price <= selectedPriceRange.max;
        
        return matchesSearch && matchesCategory && matchesPriceRange;
      });
      
      // ソート
      switch(sortBy) {
        case "priceAsc":
          filteredItems.sort((a, b) => a.price - b.price);
          break;
        case "priceDesc":
          filteredItems.sort((a, b) => b.price - a.price);
          break;
        case "ratingDesc":
          filteredItems.sort((a, b) => b.rating - a.rating);
          break;
        case "recommended":
        default:
          // おすすめ順はデフォルトのソート順を使用
          break;
      }
      
      setItems(filteredItems);
    } catch (err) {
      setError("商品の取得中にエラーが発生しました");
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, [searchQuery, selectedCategory, selectedPriceRange, sortBy]);

  // ページネーション用の計算
  const totalPages = Math.ceil(items.length / itemsPerPage);
  const displayedItems = items.slice(
    (currentPage - 1) * itemsPerPage,
    currentPage * itemsPerPage
  );

  // 検索ハンドラー
  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    // 検索実行時に1ページ目に戻す
    setCurrentPage(1);
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center h-40 w-full">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative w-full" role="alert">
        <strong className="font-bold">エラー: </strong>
        <span className="block sm:inline">{error}</span>
      </div>
    );
  }

  return (
    <div className="w-full bg-white">
      {/* ヘッダーとサーチバー */}
      <div className="mb-8 bg-yellow-400 p-4 rounded-lg w-full">
        <form onSubmit={handleSearch} className="flex flex-col sm:flex-row gap-2 w-full">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="キーワードを入力"
            className="flex-grow px-4 py-2 rounded-lg border focus:outline-none focus:ring-2 focus:ring-yellow-500"
          />
          <button 
            type="submit" 
            className="bg-red-600 hover:bg-red-700 text-white px-6 py-2 rounded-lg transition-colors"
          >
            検索
          </button>
        </form>
      </div>

      {/* フィルターとソートセクション */}
      <div className="mb-6 bg-gray-50 p-4 rounded-lg w-full">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* カテゴリフィルター */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">カテゴリ</label>
            <select
              value={selectedCategory}
              onChange={(e) => {
                setSelectedCategory(e.target.value);
                setCurrentPage(1);
              }}
              className="block w-full px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-yellow-500 focus:border-yellow-500"
            >
              {categories.map((category) => (
                <option key={category} value={category}>{category}</option>
              ))}
            </select>
          </div>
          
          {/* 価格帯フィルター */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">価格</label>
            <select
              value={selectedPriceRange.label}
              onChange={(e) => {
                const selected = priceRanges.find(range => range.label === e.target.value) || priceRanges[0];
                setSelectedPriceRange(selected);
                setCurrentPage(1);
              }}
              className="block w-full px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-yellow-500 focus:border-yellow-500"
            >
              {priceRanges.map((range) => (
                <option key={range.label} value={range.label}>{range.label}</option>
              ))}
            </select>
          </div>
          
          {/* ソートオプション */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">並び替え</label>
            <select
              value={sortBy}
              onChange={(e) => {
                setSortBy(e.target.value);
                setCurrentPage(1);
              }}
              className="block w-full px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-yellow-500 focus:border-yellow-500"
            >
              {sortOptions.map((option) => (
                <option key={option.value} value={option.value}>{option.label}</option>
              ))}
            </select>
          </div>
        </div>
      </div>
      
      {/* 検索結果サマリー */}
      <div className="mb-4 flex justify-between items-center w-full">
        <div className="text-sm text-gray-600">
          <span className="font-semibold">{items.length}</span> 件の商品が見つかりました
          {searchQuery && (
            <span>（検索キーワード: <span className="font-semibold">"{searchQuery}"</span>）</span>
          )}
        </div>
        <div className="text-sm text-gray-600">
          {currentPage} / {totalPages} ページ
        </div>
      </div>

      {/* 商品リスト */}
      {items.length === 0 ? (
        <div className="bg-gray-50 p-8 rounded-lg text-center w-full">
          <p className="text-gray-500 mb-2">検索条件に一致する商品がありません。</p>
          <p className="text-gray-500">検索条件を変更して、もう一度お試しください。</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 w-full">
          {displayedItems.map((item) => (
            <div key={item.id} className="bg-white border rounded-lg overflow-hidden shadow hover:shadow-md transition-shadow duration-300">
              <div className="relative h-48 bg-gray-100">
                <Image
                  src={item.imageUrl}
                  alt={item.name}
                  fill
                  sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 25vw"
                  style={{ objectFit: 'contain' }}
                  className="p-2"
                />
              </div>
              <div className="p-4">
                <div className="text-xs text-gray-500 mb-1">{item.brand}</div>
                <h2 className="text-sm font-medium mb-2 h-10 overflow-hidden">{item.name}</h2>
                <div className="flex items-center mb-1">
                  <div className="flex text-yellow-400">
                    {[...Array(5)].map((_, i) => (
                      <svg key={i} className={`w-4 h-4 ${i < Math.floor(item.rating) ? 'fill-current' : 'fill-gray-300'}`} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
                      </svg>
                    ))}
                  </div>
                  <span className="text-xs text-gray-500 ml-1">{item.rating.toFixed(1)}</span>
                </div>
                <div className="text-red-600 font-bold">¥{item.price.toLocaleString()}</div>
                <div className="mt-2">
                  <button className="w-full bg-red-600 hover:bg-red-700 text-white text-sm py-1 px-2 rounded transition-colors">
                    カートに入れる
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* ページネーション */}
      {totalPages > 1 && (
        <div className="flex justify-center mt-8 w-full">
          <nav className="inline-flex rounded-md shadow-sm -space-x-px" aria-label="Pagination">
            <button
              onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
              disabled={currentPage === 1}
              className={`relative inline-flex items-center px-2 py-2 rounded-l-md border ${
                currentPage === 1 
                  ? 'bg-gray-100 text-gray-400 cursor-not-allowed' 
                  : 'bg-white text-gray-500 hover:bg-gray-50'
              } text-sm font-medium`}
            >
              前へ
            </button>
            
            {[...Array(totalPages)].map((_, i) => (
              <button
                key={i}
                onClick={() => setCurrentPage(i + 1)}
                className={`relative inline-flex items-center px-4 py-2 border text-sm font-medium ${
                  currentPage === i + 1
                    ? 'z-10 bg-yellow-50 border-yellow-500 text-yellow-600'
                    : 'bg-white border-gray-300 text-gray-500 hover:bg-gray-50'
                }`}
              >
                {i + 1}
              </button>
            ))}
            
            <button
              onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
              disabled={currentPage === totalPages}
              className={`relative inline-flex items-center px-2 py-2 rounded-r-md border ${
                currentPage === totalPages 
                  ? 'bg-gray-100 text-gray-400 cursor-not-allowed' 
                  : 'bg-white text-gray-500 hover:bg-gray-50'
              } text-sm font-medium`}
            >
              次へ
            </button>
          </nav>
        </div>
      )}
    </div>
  );
}
