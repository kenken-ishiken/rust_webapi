import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // output: 'export', // 静的エクスポート設定を削除
  // distDir: 'out'    // カスタム出力ディレクトリ設定を削除
  
  images: {
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'placehold.jp',
        pathname: '**',
      },
    ],
  },
};

export default nextConfig;
