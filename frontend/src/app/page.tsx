import Link from "next/link";

export default function Home() {
  return (
    <section className="grid gap-4 rounded-md border bg-white p-6 shadow-sm">
      <h2 className="text-xl font-semibold">テスト用フロントエンド</h2>
      <p className="text-sm leading-7 text-gray-700">
        問い合わせAPIの動作確認を行うための Next.js 実装です。
      </p>
      <Link className="text-blue-600 underline" href="/inquiry">
        問い合わせページへ
      </Link>
    </section>
  );
}
