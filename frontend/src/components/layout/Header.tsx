"use client";

import Link from "next/link";
import { useState } from "react";

type HeaderProps = {
  title: string;
};

export function Header({ title }: HeaderProps) {
  const [open, setOpen] = useState<boolean>(false);

  return (
    <header className="sticky top-0 z-40 border-b bg-white">
      <div className="mx-auto flex h-14 max-w-5xl items-center px-4">
        <button
          type="button"
          className="rounded p-2 hover:bg-gray-100"
          aria-label="メニューを開く"
          onClick={() => setOpen((value: boolean) => !value)}
        >
          ☰
        </button>
        <h1 className="mx-4 flex-1 truncate text-center text-lg font-semibold">{title}</h1>
        <span className="w-8" aria-hidden="true" />
      </div>
      {open ? (
        <nav className="border-t bg-white">
          <ul className="mx-auto flex max-w-5xl flex-col px-4 py-2">
            <li>
              <Link className="block rounded px-2 py-2 hover:bg-gray-100" href="/">
                トップ
              </Link>
            </li>
            <li>
              <Link className="block rounded px-2 py-2 hover:bg-gray-100" href="/inquiry">
                問い合わせ
              </Link>
            </li>
          </ul>
        </nav>
      ) : null}
    </header>
  );
}
