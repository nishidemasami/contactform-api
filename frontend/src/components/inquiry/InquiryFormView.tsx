import { useEffect, useRef } from "react";

import {
  clampToCodePoints,
  INQUIRY_MESSAGE_MAX_CODE_POINTS,
  INQUIRY_NAME_MAX_CODE_POINTS,
  validateEmailField,
  validateInquiryDraft,
  validateMessageField,
  validateNameField,
} from "../../lib/inquiryValidation";

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
  const nameInputRef = useRef<HTMLInputElement | null>(null);
  const emailInputRef = useRef<HTMLInputElement | null>(null);
  const messageInputRef = useRef<HTMLTextAreaElement | null>(null);
  const isComposingNameRef = useRef<boolean>(false);
  const isComposingMessageRef = useRef<boolean>(false);

  useEffect(() => {
    nameInputRef.current?.setCustomValidity(validateNameField(name) ?? "");
  }, [name]);

  useEffect(() => {
    emailInputRef.current?.setCustomValidity(validateEmailField(email) ?? "");
  }, [email]);

  useEffect(() => {
    messageInputRef.current?.setCustomValidity(validateMessageField(message) ?? "");
  }, [message]);

  const handleNameChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    const rawValue: string = event.target.value;
    const isComposing: boolean =
      isComposingNameRef.current ||
      (event.nativeEvent instanceof InputEvent && event.nativeEvent.isComposing);
    const nextValue: string = isComposing
      ? rawValue
      : clampToCodePoints(rawValue, INQUIRY_NAME_MAX_CODE_POINTS);

    event.currentTarget.setCustomValidity(validateNameField(nextValue) ?? "");
    onNameChange(nextValue);
  };

  const handleNameCompositionEnd = (
    event: React.CompositionEvent<HTMLInputElement>,
  ): void => {
    isComposingNameRef.current = false;
    const clampedValue: string = clampToCodePoints(
      event.currentTarget.value,
      INQUIRY_NAME_MAX_CODE_POINTS,
    );
    event.currentTarget.setCustomValidity(validateNameField(clampedValue) ?? "");
    if (clampedValue !== name) {
      onNameChange(clampedValue);
    }
  };

  const handleEmailChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    const nextValue: string = event.target.value;
    event.currentTarget.setCustomValidity(validateEmailField(nextValue) ?? "");
    onEmailChange(nextValue);
  };

  const handleMessageChange = (
    event: React.ChangeEvent<HTMLTextAreaElement>,
  ): void => {
    const rawValue: string = event.target.value;
    const isComposing: boolean =
      isComposingMessageRef.current ||
      (event.nativeEvent instanceof InputEvent && event.nativeEvent.isComposing);
    const nextValue: string = isComposing
      ? rawValue
      : clampToCodePoints(rawValue, INQUIRY_MESSAGE_MAX_CODE_POINTS);

    event.currentTarget.setCustomValidity(validateMessageField(nextValue) ?? "");
    onMessageChange(nextValue);
  };

  const handleMessageCompositionEnd = (
    event: React.CompositionEvent<HTMLTextAreaElement>,
  ): void => {
    isComposingMessageRef.current = false;
    const clampedValue: string = clampToCodePoints(
      event.currentTarget.value,
      INQUIRY_MESSAGE_MAX_CODE_POINTS,
    );
    event.currentTarget.setCustomValidity(validateMessageField(clampedValue) ?? "");
    if (clampedValue !== message) {
      onMessageChange(clampedValue);
    }
  };

  const handleFormSubmit = (event: React.FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    if (isSubmitting || !acceptedTerms) {
      return;
    }

    const validation = validateInquiryDraft({ name, email, message });
    nameInputRef.current?.setCustomValidity(validation.nameError ?? "");
    emailInputRef.current?.setCustomValidity(validation.emailError ?? "");
    messageInputRef.current?.setCustomValidity(validation.messageError ?? "");

    if (!event.currentTarget.reportValidity() || !validation.isValid) {
      return;
    }

    onSubmit();
  };

  return (
    <section className="rounded-md border bg-white p-5 shadow-sm">
      <form className="grid gap-4" onSubmit={handleFormSubmit}>
        <label className="grid gap-1">
          <span className="text-sm font-medium">氏名</span>
          <input
            ref={nameInputRef}
            className="rounded border px-3 py-2"
            type="text"
            required
            value={name}
            onCompositionStart={() => {
              isComposingNameRef.current = true;
            }}
            onCompositionEnd={handleNameCompositionEnd}
            onChange={handleNameChange}
          />
        </label>

        <label className="grid gap-1">
          <span className="text-sm font-medium">連絡先（メールアドレス）</span>
          <input
            ref={emailInputRef}
            className="rounded border px-3 py-2"
            type="text"
            inputMode="email"
            autoComplete="email"
            required
            value={email}
            onChange={handleEmailChange}
          />
        </label>

        <label className="grid gap-1">
          <span className="text-sm font-medium">本文</span>
          <textarea
            ref={messageInputRef}
            className="min-h-36 rounded border px-3 py-2"
            required
            value={message}
            onCompositionStart={() => {
              isComposingMessageRef.current = true;
            }}
            onCompositionEnd={handleMessageCompositionEnd}
            onChange={handleMessageChange}
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
          type="submit"
          className="rounded bg-blue-600 px-4 py-2 font-semibold text-white disabled:bg-gray-300"
          disabled={isSubmitting || !acceptedTerms}
        >
          投稿
        </button>
      </form>
    </section>
  );
}
