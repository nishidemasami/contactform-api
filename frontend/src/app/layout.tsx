import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Footer } from "@/components/layout/Footer";
import { Header } from "@/components/layout/Header";
import "./globals.css";

export const metadata: Metadata = {
  title: "contactform-api frontend",
  description: "問い合わせフォーム検証用のフロントエンド",
};

type RootLayoutProps = {
  children: ReactNode;
};

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="ja">
      <body className="min-h-screen bg-gray-50 text-gray-900">
        <div className="flex min-h-screen flex-col">
          <Header title="contactform-api frontend" />
          <main className="mx-auto w-full max-w-5xl flex-1 px-4 py-6">{children}</main>
          <Footer />
        </div>
      </body>
    </html>
  );
}
