export const INQUIRY_NAME_MAX_CODE_POINTS: number = 100;
export const INQUIRY_EMAIL_MAX_CODE_POINTS: number = 254;
export const INQUIRY_EMAIL_LOCAL_MAX_CODE_POINTS: number = 64;
export const INQUIRY_EMAIL_DOMAIN_MAX_CODE_POINTS: number = 253;
export const INQUIRY_EMAIL_LABEL_MAX_CODE_POINTS: number = 63;
export const INQUIRY_MESSAGE_MAX_CODE_POINTS: number = 5000;

export type InquiryDraft = {
  name: string;
  email: string;
  message: string;
};

export type InquiryValidationResult = {
  isValid: boolean;
  nameError: string | null;
  emailError: string | null;
  messageError: string | null;
};

/**
 * Rust `char::is_whitespace`（Unicode `White_Space` プロパティ全25コードポイント）と同一の空白判定
 */
export function isUnicodeWhitespaceCodePoint(cp: number): boolean {
  return (
    (cp >= 0x0009 && cp <= 0x000d) ||
    cp === 0x0020 ||
    cp === 0x0085 ||
    cp === 0x00a0 ||
    cp === 0x1680 ||
    (cp >= 0x2000 && cp <= 0x200a) ||
    cp === 0x2028 ||
    cp === 0x2029 ||
    cp === 0x202f ||
    cp === 0x205f ||
    cp === 0x3000
  );
}

function isControlOrLoneSurrogate(cp: number): boolean {
  return (
    cp <= 0x001f ||
    (cp >= 0x007f && cp <= 0x009f) ||
    (cp >= 0xd800 && cp <= 0xdfff)
  );
}

/**
 * UTF-16 コード単位ではなく Unicode コードポイント数（スカラー値単位）を返す
 */
export function countCodePoints(value: string): number {
  return Array.from(value).length;
}

/**
 * サロゲートペアを破壊せず、先頭から最大 `maxCodePoints` コードポイントに切り詰める
 */
export function clampToCodePoints(value: string, maxCodePoints: number): string {
  const codePoints: string[] = Array.from(value);
  if (codePoints.length <= maxCodePoints) {
    return value;
  }
  return codePoints.slice(0, maxCodePoints).join("");
}

/**
 * ドメイン層の `trim_utf8_range` と同一の Unicode `White_Space` 基準で前後空白を除去する
 */
export function trimUnicodeWhitespace(value: string): string {
  const chars: string[] = Array.from(value);
  let start: number = 0;
  while (start < chars.length) {
    const cp: number = chars[start].codePointAt(0) ?? 0;
    if (!isUnicodeWhitespaceCodePoint(cp)) {
      break;
    }
    start += 1;
  }

  let end: number = chars.length;
  while (end > start) {
    const cp: number = chars[end - 1].codePointAt(0) ?? 0;
    if (!isUnicodeWhitespaceCodePoint(cp)) {
      break;
    }
    end -= 1;
  }

  return chars.slice(start, end).join("");
}

function isAsciiAlpha(cp: number): boolean {
  return (cp >= 0x41 && cp <= 0x5a) || (cp >= 0x61 && cp <= 0x7a);
}

function isAsciiDigit(cp: number): boolean {
  return cp >= 0x30 && cp <= 0x39;
}

function isEmailAtext(cp: number): boolean {
  if (isAsciiAlpha(cp) || isAsciiDigit(cp)) {
    return true;
  }
  switch (cp) {
    case 0x21: // !
    case 0x23: // #
    case 0x24: // $
    case 0x25: // %
    case 0x26: // &
    case 0x27: // '
    case 0x2a: // *
    case 0x2b: // +
    case 0x2d: // -
    case 0x2f: // /
    case 0x3d: // =
    case 0x3f: // ?
    case 0x5e: // ^
    case 0x5f: // _
    case 0x60: // `
    case 0x7b: // {
    case 0x7c: // |
    case 0x7d: // }
    case 0x7e: // ~
      return true;
    default:
      return false;
  }
}

function isValidEmailLocalPart(localChars: string[]): boolean {
  const len: number = localChars.length;
  if (len === 0 || len > INQUIRY_EMAIL_LOCAL_MAX_CODE_POINTS) {
    return false;
  }
  const firstCp: number = localChars[0].codePointAt(0) ?? 0;
  const lastCp: number = localChars[len - 1].codePointAt(0) ?? 0;
  if (firstCp === 0x2e || lastCp === 0x2e) {
    return false;
  }

  let prevWasDot: boolean = false;
  for (const ch of localChars) {
    const cp: number = ch.codePointAt(0) ?? 0;
    if (cp === 0x2e) {
      if (prevWasDot) {
        return false;
      }
      prevWasDot = true;
    } else if (isEmailAtext(cp)) {
      prevWasDot = false;
    } else {
      return false;
    }
  }
  return true;
}

function isValidEmailDomainPart(domainChars: string[]): boolean {
  const len: number = domainChars.length;
  if (len < 4 || len > INQUIRY_EMAIL_DOMAIN_MAX_CODE_POINTS) {
    return false;
  }
  const firstCp: number = domainChars[0].codePointAt(0) ?? 0;
  if (firstCp === 0x2e || firstCp === 0x2d) {
    return false;
  }

  let dotCount: number = 0;
  let labelLen: number = 0;
  let labelHasAlpha: boolean = false;
  let prevWasHyphen: boolean = false;

  for (const ch of domainChars) {
    const cp: number = ch.codePointAt(0) ?? 0;
    if (cp === 0x2e) {
      if (labelLen === 0 || prevWasHyphen) {
        return false;
      }
      dotCount += 1;
      labelLen = 0;
      labelHasAlpha = false;
      prevWasHyphen = false;
    } else if (isAsciiAlpha(cp)) {
      if (labelLen >= INQUIRY_EMAIL_LABEL_MAX_CODE_POINTS) {
        return false;
      }
      labelLen += 1;
      labelHasAlpha = true;
      prevWasHyphen = false;
    } else if (isAsciiDigit(cp)) {
      if (labelLen >= INQUIRY_EMAIL_LABEL_MAX_CODE_POINTS) {
        return false;
      }
      labelLen += 1;
      prevWasHyphen = false;
    } else if (cp === 0x2d) {
      if (labelLen === 0 || labelLen >= INQUIRY_EMAIL_LABEL_MAX_CODE_POINTS) {
        return false;
      }
      labelLen += 1;
      prevWasHyphen = true;
    } else {
      return false;
    }
  }

  return dotCount >= 1 && labelLen >= 2 && !prevWasHyphen && labelHasAlpha;
}

/**
 * トリム済みメールアドレス文字列がドメイン層と同じ RFC 5321/5322/1123 構文を満たすか判定する
 */
export function isValidTrimmedEmailSyntax(trimmed: string): boolean {
  const chars: string[] = Array.from(trimmed);
  if (chars.length < 6 || chars.length > INQUIRY_EMAIL_MAX_CODE_POINTS) {
    return false;
  }

  let atPos: number = -1;
  for (let i: number = 0; i < chars.length; i += 1) {
    const cp: number = chars[i].codePointAt(0) ?? 0;
    if (cp < 0x21 || cp > 0x7e) {
      return false;
    }
    if (cp === 0x40) {
      if (atPos !== -1) {
        return false;
      }
      atPos = i;
    }
  }

  if (atPos < 1) {
    return false;
  }

  const localChars: string[] = chars.slice(0, atPos);
  const domainChars: string[] = chars.slice(atPos + 1);
  return isValidEmailLocalPart(localChars) && isValidEmailDomainPart(domainChars);
}

export function validateNameField(value: string): string | null {
  const chars: string[] = Array.from(value);
  let hasNonWhitespace: boolean = false;
  let hasDisallowedControl: boolean = false;

  for (const ch of chars) {
    const cp: number = ch.codePointAt(0) ?? 0;
    if (!isUnicodeWhitespaceCodePoint(cp)) {
      hasNonWhitespace = true;
    }
    if (isControlOrLoneSurrogate(cp)) {
      hasDisallowedControl = true;
    }
  }

  if (!hasNonWhitespace) {
    return "氏名を入力してください。";
  }
  if (chars.length > INQUIRY_NAME_MAX_CODE_POINTS) {
    return `氏名は${INQUIRY_NAME_MAX_CODE_POINTS}文字以内で入力してください。`;
  }
  if (hasDisallowedControl) {
    return "氏名に使用できない制御文字が含まれています。";
  }
  return null;
}

export function validateEmailField(value: string): string | null {
  const trimmed: string = trimUnicodeWhitespace(value);
  const trimmedLen: number = countCodePoints(trimmed);

  if (trimmedLen === 0) {
    return "連絡先（メールアドレス）を入力してください。";
  }
  if (trimmedLen > INQUIRY_EMAIL_MAX_CODE_POINTS) {
    return `連絡先（メールアドレス）は前後の空白を除き${INQUIRY_EMAIL_MAX_CODE_POINTS}文字以内で入力してください。`;
  }
  if (!isValidTrimmedEmailSyntax(trimmed)) {
    return "有効なメールアドレス形式で入力してください。";
  }
  return null;
}

export function validateMessageField(value: string): string | null {
  const chars: string[] = Array.from(value);
  let hasNonWhitespace: boolean = false;
  let hasDisallowedControl: boolean = false;

  for (const ch of chars) {
    const cp: number = ch.codePointAt(0) ?? 0;
    if (!isUnicodeWhitespaceCodePoint(cp)) {
      hasNonWhitespace = true;
    }
    if (
      cp !== 0x09 &&
      cp !== 0x0a &&
      cp !== 0x0d &&
      isControlOrLoneSurrogate(cp)
    ) {
      hasDisallowedControl = true;
    }
  }

  if (!hasNonWhitespace) {
    return "本文を入力してください。";
  }
  if (chars.length > INQUIRY_MESSAGE_MAX_CODE_POINTS) {
    return `本文は${INQUIRY_MESSAGE_MAX_CODE_POINTS}文字以内で入力してください。`;
  }
  if (hasDisallowedControl) {
    return "本文に使用できない制御文字が含まれています。";
  }
  return null;
}

export function validateInquiryDraft(draft: InquiryDraft): InquiryValidationResult {
  const nameError: string | null = validateNameField(draft.name);
  const emailError: string | null = validateEmailField(draft.email);
  const messageError: string | null = validateMessageField(draft.message);

  return {
    isValid: nameError === null && emailError === null && messageError === null,
    nameError,
    emailError,
    messageError,
  };
}

export function canSubmitInquiry(
  draft: InquiryDraft,
  acceptedTerms: boolean,
  isSubmitting: boolean,
): boolean {
  if (isSubmitting || !acceptedTerms) {
    return false;
  }
  return validateInquiryDraft(draft).isValid;
}
