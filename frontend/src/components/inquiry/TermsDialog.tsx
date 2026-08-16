"use client";

type TermsDialogProps = {
  isOpen: boolean;
  onClose: () => void;
};

const TERMS_TEXT: string = `送信された情報はセキュリティ規格に関する国際規格であるISO/27001に準拠しているサービス(AWS、CloudFlare、GitLab、Slack、GitHub Actions、Google Cloud Platform)のみを利用して取り扱い、万全のセキュリティ体制により適切に取り扱いますが、このサービスを利用したことに起因して何かしらの損害が発生しても、直接損害か間接損害か否か、予見できたか否かを問わず、一切の責任を負いません。
送信された内容は、このウェブサイトの運営のため、日本語ドキュメントによる技術的知見の普及のため、および送信者との連絡のためにのみ使用します。
送信された内容は、職務上守秘義務を負う情報とはみなされません。
送信された内容によって、いかなる契約も成立することはありません。
IPアドレスなどの送信元情報をログとして保存しており、違法性がある場合または法令に基づく場合に限り、セキュリティ団体や捜査機関などに情報を提供する場合があります。
AIおよび人力によるスパムフィルタによりフィルタリングを実施しており、もしスパムでなくても送信された内容がスパムと誤判定されて届かないことがあります。また、届いたお問い合わせに対して必ず返信することを約束するものではありません。
以上についてご了承下さい。`;

export function TermsDialog({ isOpen, onClose }: TermsDialogProps) {
  if (!isOpen) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 px-4"
      onClick={onClose}
      role="presentation"
    >
      <div
        className="max-h-[80vh] w-full max-w-2xl rounded-md bg-white p-4 shadow-xl"
        onClick={(event: React.MouseEvent<HTMLDivElement>) => event.stopPropagation()}
        onKeyDown={(event: React.KeyboardEvent<HTMLDivElement>) => {
          if (event.key === "Escape") {
            onClose();
          }
        }}
        role="dialog"
        aria-modal="true"
        aria-labelledby="terms-dialog-title"
      >
        <div className="mb-3 flex items-center justify-between">
          <h2 id="terms-dialog-title" className="text-lg font-semibold">
            利用規約
          </h2>
          <button
            type="button"
            className="rounded px-2 text-gray-500 hover:bg-gray-100"
            aria-label="利用規約を閉じる"
            onClick={onClose}
            autoFocus
          >
            ×
          </button>
        </div>
        <p className="whitespace-pre-line text-sm leading-7 text-gray-700">{TERMS_TEXT}</p>
      </div>
    </div>
  );
}
