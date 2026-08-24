import type { Meta, StoryObj } from "@storybook/react";
import { TermsDialog } from "./TermsDialog";

const meta: Meta<typeof TermsDialog> = {
  title: "Inquiry/TermsDialog",
  component: TermsDialog,
  tags: ["autodocs"],
  argTypes: {
    onClose: { action: "closed" },
  },
  parameters: { layout: "fullscreen" },
  // 👇 プレビュー領域に最低限の高さを確保するデコレーターを追加
  decorators: [
    (Story) => (
      <div className="relative min-h-[800px] w-full">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof TermsDialog>;

export const Open: Story = {
  args: {
    isOpen: true,
  },
};

export const Closed: Story = {
  args: {
    isOpen: false,
  },
};
