"use client";

type ToastProps = {
  message: string;
  tone: "success" | "error";
  onClose: () => void;
};

export function Toast({ message, tone, onClose }: ToastProps) {
  return (
    <div className="fixed right-4 bottom-4 z-50 w-full max-w-sm rounded-md border bg-white p-4 shadow-lg">
      <div className="flex items-start justify-between gap-3">
        <p className={tone === "success" ? "text-green-700" : "text-red-700"}>{message}</p>
        <button
          type="button"
          className="rounded px-2 text-gray-500 hover:bg-gray-100"
          onClick={onClose}
          aria-label="トーストを閉じる"
        >
          ×
        </button>
      </div>
    </div>
  );
}
