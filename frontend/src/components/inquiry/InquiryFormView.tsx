type InquiryFormViewProps = {
  name: string;
  email: string;
  message: string;
  acceptedTerms: boolean;
  isSubmitting: boolean;
  onNameChange: (value: string) => void;
  onEmailChange: (value: string) => void;
  onMessageChange: (value: string) => void;
  onAcceptedTermsChange: (value: boolean) => void;
  onSubmit: () => void;
  onOpenTerms: () => void;
};

export function InquiryFormView({
  name,
  email,
  message,
  acceptedTerms,
  isSubmitting,
  onNameChange,
  onEmailChange,
  onMessageChange,
  onAcceptedTermsChange,
  onSubmit,
  onOpenTerms,
}: InquiryFormViewProps) {
  return (
    <section className="rounded-md border bg-white p-5 shadow-sm">
      <div className="grid gap-4">
        <label className="grid gap-1">
          <span className="text-sm font-medium">氏名</span>
          <input
            className="rounded border px-3 py-2"
            type="text"
            value={name}
            onChange={(event: React.ChangeEvent<HTMLInputElement>) => onNameChange(event.target.value)}
          />
        </label>

        <label className="grid gap-1">
          <span className="text-sm font-medium">連絡先（メールアドレス）</span>
          <input
            className="rounded border px-3 py-2"
            type="email"
            value={email}
            onChange={(event: React.ChangeEvent<HTMLInputElement>) => onEmailChange(event.target.value)}
          />
        </label>

        <label className="grid gap-1">
          <span className="text-sm font-medium">本文</span>
          <textarea
            className="min-h-36 rounded border px-3 py-2"
            value={message}
            onChange={(event: React.ChangeEvent<HTMLTextAreaElement>) => onMessageChange(event.target.value)}
          />
        </label>

        <label className="flex items-start gap-2 text-sm">
          <input
            type="checkbox"
            checked={acceptedTerms}
            onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
              onAcceptedTermsChange(event.target.checked)
            }
          />
          <span>
            利用規約に同意する（
            <button type="button" className="text-blue-600 underline" onClick={onOpenTerms}>
              利用規約
            </button>
            ）
          </span>
        </label>

        <button
          type="button"
          className="rounded bg-blue-600 px-4 py-2 font-semibold text-white disabled:bg-gray-300"
          onClick={onSubmit}
          disabled={isSubmitting || !acceptedTerms}
        >
          投稿
        </button>
      </div>
    </section>
  );
}
