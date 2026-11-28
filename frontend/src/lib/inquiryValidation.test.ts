import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { InquiryFormView } from "../components/inquiry/InquiryFormView";
import {
  canSubmitInquiry,
  clampToCodePoints,
  countCodePoints,
  INQUIRY_EMAIL_MAX_CODE_POINTS,
  INQUIRY_MESSAGE_MAX_CODE_POINTS,
  INQUIRY_NAME_MAX_CODE_POINTS,
  trimUnicodeWhitespace,
  validateEmailField,
  validateInquiryDraft,
  validateMessageField,
  validateNameField,
} from "./inquiryValidation";

describe("Unicodeコードポイント単位の文字数計測・切り詰め", () => {
  it("サロゲートペア（𠮷）51文字はUTF-16では102単位だが51コードポイントとして有効になる", () => {
    const fiftyOneYoshinoya: string = "𠮷".repeat(51);
    expect(fiftyOneYoshinoya.length).toBe(102);
    expect(countCodePoints(fiftyOneYoshinoya)).toBe(51);
    expect(clampToCodePoints(fiftyOneYoshinoya, INQUIRY_NAME_MAX_CODE_POINTS)).toBe(
      fiftyOneYoshinoya,
    );
    expect(validateNameField(fiftyOneYoshinoya)).toBeNull();
  });

  it("サロゲートペア100文字の氏名は有効、101文字は100コードポイントに切り詰め・検証される", () => {
    const hundred: string = "𠮷".repeat(100);
    const hundredAndOne: string = "𠮷".repeat(101);
    expect(hundred.length).toBe(200);
    expect(validateNameField(hundred)).toBeNull();
    expect(validateNameField(hundredAndOne)).not.toBeNull();

    const clamped: string = clampToCodePoints(
      hundredAndOne,
      INQUIRY_NAME_MAX_CODE_POINTS,
    );
    expect(countCodePoints(clamped)).toBe(100);
    expect(clamped).toBe(hundred);
  });

  it("サロゲートペア5000文字の本文は有効、5001文字は5000コードポイントに切り詰め・検証される", () => {
    const fiveThousand: string = "𠮷".repeat(5000);
    const fiveThousandAndOne: string = "𠮷".repeat(5001);
    expect(fiveThousand.length).toBe(10000);
    expect(validateMessageField(fiveThousand)).toBeNull();
    expect(validateMessageField(fiveThousandAndOne)).not.toBeNull();

    const clamped: string = clampToCodePoints(
      fiveThousandAndOne,
      INQUIRY_MESSAGE_MAX_CODE_POINTS,
    );
    expect(countCodePoints(clamped)).toBe(5000);
    expect(clamped).toBe(fiveThousand);
  });
});

describe("メールアドレスのtrim正規化とRFC構文検証", () => {
  it("有効な254文字のメールアドレスに前後空白が付いた入力を受理する", () => {
    const local: string = "a".repeat(64);
    const label1: string = "b".repeat(63);
    const label2: string = "c".repeat(63);
    const label3: string = "d".repeat(58);
    const email254: string = `${local}@${label1}.${label2}.${label3}.co`;
    expect(countCodePoints(email254)).toBe(INQUIRY_EMAIL_MAX_CODE_POINTS);

    const padded: string = `  \u3000${email254}\t\n `;
    expect(trimUnicodeWhitespace(padded)).toBe(email254);
    expect(validateEmailField(padded)).toBeNull();
  });

  it("trim後に255文字のメールアドレスや連続ドットを拒否する", () => {
    const local: string = "a".repeat(64);
    const label1: string = "b".repeat(63);
    const label2: string = "c".repeat(63);
    const label3: string = "d".repeat(59);
    const email255: string = `${local}@${label1}.${label2}.${label3}.co`;
    expect(countCodePoints(email255)).toBe(255);

    expect(validateEmailField(email255)).not.toBeNull();
    expect(validateEmailField("a..b@example.com")).not.toBeNull();
    expect(validateEmailField("user@localhost")).not.toBeNull();
  });
});

describe("フォーム送信ガードとHTML構造", () => {
  it("同意チェックのみオンで入力項目が空・空白のみの場合は送信不可となる", () => {
    expect(
      canSubmitInquiry({ name: "", email: "", message: "" }, true, false),
    ).toBe(false);
    expect(
      canSubmitInquiry(
        { name: "   ", email: "user@example.com", message: "本文" },
        true,
        false,
      ),
    ).toBe(false);
    expect(
      canSubmitInquiry(
        { name: "山田 太郎", email: "user@example.com", message: "本文" },
        false,
        false,
      ),
    ).toBe(false);
    expect(
      canSubmitInquiry(
        { name: "山田 太郎", email: "user@example.com", message: "本文" },
        true,
        true,
      ),
    ).toBe(false);
    expect(
      canSubmitInquiry(
        { name: "山田 太郎", email: "  user@example.com  ", message: "本文\n改行あり" },
        true,
        false,
      ),
    ).toBe(true);
  });

  it("氏名・本文はtrim前の文字数で上限判定される", () => {
    const paddedOverLimitName: string = `  ${"あ".repeat(99)}`;
    expect(countCodePoints(paddedOverLimitName)).toBe(101);
    const res = validateInquiryDraft({
      name: paddedOverLimitName,
      email: "user@example.com",
      message: "本文",
    });
    expect(res.isValid).toBe(false);
    expect(res.nameError).not.toBeNull();
  });

  it("InquiryFormViewは<form>とtype='submit'を使用し、UTF-16のmaxlength属性を持たない", () => {
    const html: string = renderToStaticMarkup(
      createElement(InquiryFormView, {
        name: "",
        email: "",
        message: "",
        acceptedTerms: true,
        isSubmitting: false,
        onNameChange: () => {},
        onEmailChange: () => {},
        onMessageChange: () => {},
        onAcceptedTermsChange: () => {},
        onSubmit: () => {},
        onOpenTerms: () => {},
      }),
    );

    expect(html).toContain("<form");
    expect(html).toContain('type="submit"');
    expect(html).not.toContain("maxlength=");
  });
});
