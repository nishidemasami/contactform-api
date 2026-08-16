import { describe, expect, it, vi } from "vitest";

import { submitInquiry } from "./inquiryApi";

describe("submitInquiry", () => {
  it("POST /api/v1/inquiry に JSON を送信する", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue({ ok: true } as Response);

    await submitInquiry({ name: "A", email: "a@example.com", message: "hello" });

    expect(fetchMock).toHaveBeenCalledWith("/api/v1/inquiry", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: "A", email: "a@example.com", message: "hello" }),
    });

    fetchMock.mockRestore();
  });

  it("レスポンスが失敗時は例外を送出する", async () => {
    const fetchMock = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue({ ok: false } as Response);

    await expect(
      submitInquiry({ name: "A", email: "a@example.com", message: "hello" }),
    ).rejects.toThrow("inquiry request failed");

    fetchMock.mockRestore();
  });
});
