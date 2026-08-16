export type InquiryRequest = {
  name: string;
  email: string;
  message: string;
};

export async function submitInquiry(request: InquiryRequest): Promise<void> {
  const response: Response = await fetch("/api/v1/inquiry", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    throw new Error("inquiry request failed");
  }
}
