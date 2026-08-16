"use client";

import { useState } from "react";

import { Toast } from "@/components/common/Toast";
import { submitInquiry } from "@/lib/inquiryApi";
import { InquiryFormView } from "./InquiryFormView";
import { TermsDialog } from "./TermsDialog";

type ToastState = {
  message: string;
  tone: "success" | "error";
};

export function InquiryFormContainer() {
  const [name, setName] = useState<string>("");
  const [email, setEmail] = useState<string>("");
  const [message, setMessage] = useState<string>("");
  const [acceptedTerms, setAcceptedTerms] = useState<boolean>(false);
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);
  const [isTermsOpen, setIsTermsOpen] = useState<boolean>(false);
  const [toast, setToast] = useState<ToastState | null>(null);

  const onSubmit = async (): Promise<void> => {
    setIsSubmitting(true);
    try {
      await submitInquiry({ name, email, message });
      setToast({ message: "問い合わせを送信しました。", tone: "success" });
      setName("");
      setEmail("");
      setMessage("");
    } catch {
      setToast({ message: "問い合わせ送信に失敗しました。", tone: "error" });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <>
      <InquiryFormView
        name={name}
        email={email}
        message={message}
        acceptedTerms={acceptedTerms}
        isSubmitting={isSubmitting}
        onNameChange={setName}
        onEmailChange={setEmail}
        onMessageChange={setMessage}
        onAcceptedTermsChange={setAcceptedTerms}
        onSubmit={onSubmit}
        onOpenTerms={() => setIsTermsOpen(true)}
      />
      <TermsDialog isOpen={isTermsOpen} onClose={() => setIsTermsOpen(false)} />
      {toast ? (
        <Toast message={toast.message} tone={toast.tone} onClose={() => setToast(null)} />
      ) : null}
    </>
  );
}
