import type { Meta, StoryObj } from "@storybook/react";
import { InquiryFormView } from "./InquiryFormView";

const meta: Meta<typeof InquiryFormView> = {
  title: "Inquiry/InquiryFormView",
  component: InquiryFormView,
  tags: ["autodocs"],
  argTypes: {
    onNameChange: { action: "nameChanged" },
    onEmailChange: { action: "emailChanged" },
    onMessageChange: { action: "messageChanged" },
    onAcceptedTermsChange: { action: "acceptedTermsChanged" },
    onSubmit: { action: "submitted" },
    onOpenTerms: { action: "openTerms" },
  },
};

export default meta;
type Story = StoryObj<typeof InquiryFormView>;

export const Default: Story = {
  args: {
    name: "",
    email: "",
    message: "",
    acceptedTerms: false,
    isSubmitting: false,
  },
};

export const FilledAndAgreed: Story = {
  args: {
    name: "山田 太郎",
    email: "taro.yamada@example.com",
    message: "お問い合わせ内容のサンプルテキストです。",
    acceptedTerms: true,
    isSubmitting: false,
  },
};

export const Submitting: Story = {
  args: {
    name: "山田 太郎",
    email: "taro.yamada@example.com",
    message: "送信処理中のテキストです。",
    acceptedTerms: true,
    isSubmitting: true,
  },
};
