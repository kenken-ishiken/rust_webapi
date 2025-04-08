import ProductList from './components/ProductList';

export default function Home() {
  return (
    <div className="min-h-screen p-8">
      <header className="mb-8">
        <h1 className="text-3xl font-bold text-center">商品情報一覧</h1>
      </header>
      <main>
        <ProductList />
      </main>
      <footer className="mt-12 text-center text-gray-500 text-sm">
        <p>© 2025 商品情報一覧アプリ</p>
      </footer>
    </div>
  );
}
