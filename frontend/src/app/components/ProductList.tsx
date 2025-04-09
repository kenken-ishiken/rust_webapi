'use client';

import { useState } from 'react';

interface Item {
  id: number;
  name: string;
  description: string | null;
}

const mockItems: Item[] = [
  {
    id: 1,
    name: "商品A",
    description: "これは商品Aの説明です。高品質な素材を使用しています。"
  },
  {
    id: 2,
    name: "商品B",
    description: "商品Bは耐久性に優れています。"
  },
  {
    id: 3,
    name: "商品C",
    description: null
  },
  {
    id: 4,
    name: "商品D",
    description: "商品Dは最新のテクノロジーを搭載しています。"
  },
  {
    id: 5,
    name: "商品E",
    description: "軽量で持ち運びに便利な商品Eです。"
  }
];

export default function ProductList() {
  const [items] = useState<Item[]>(mockItems);
  const [loading] = useState(false);
  const [error] = useState<string | null>(null);

  if (loading) {
    return (
      <div className="flex justify-center items-center h-40">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert">
        <strong className="font-bold">エラー: </strong>
        <span className="block sm:inline">{error}</span>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4">
      <h1 className="text-2xl font-bold mb-6">商品一覧</h1>
      
      {items.length === 0 ? (
        <p className="text-gray-500">商品がありません。</p>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {items.map((item) => (
            <div key={item.id} className="border rounded-lg overflow-hidden shadow-lg hover:shadow-xl transition-shadow duration-300">
              <div className="p-6">
                <h2 className="text-xl font-semibold mb-2">{item.name}</h2>
                {item.description ? (
                  <p className="text-gray-600">{item.description}</p>
                ) : (
                  <p className="text-gray-400 italic">説明なし</p>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
