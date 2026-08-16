import { InquiryFormContainer } from "@/components/inquiry/InquiryFormContainer";

export default function InquiryPage() {
  return (
    <div className="grid gap-4">
      <h2 className="text-xl font-semibold">問い合わせページ</h2>
      <InquiryFormContainer />
    </div>
  );
}
