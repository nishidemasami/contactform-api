import type { Meta, StoryObj } from "@storybook/react";
import { Toast } from "./Toast";

const meta: Meta<typeof Toast> = {
  title: "Common/Toast",
  component: Toast,
  tags: ["autodocs"],
  argTypes: {
    tone: {
      control: "radio",
      options: ["success", "error"],
    },
    onClose: { action: "closed" },
  },
  parameters: { layout: "fullscreen" },
  // 👇 プレビュー領域に最低限の高さを確保するデコレーターを追加
  decorators: [
    (Story) => (
      <div className="relative min-h-[200px] w-full">
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof Toast>;

export const Success: Story = {
  args: {
    message: "お問い合わせを送信しました。",
    tone: "success",
  },
};

export const Error: Story = {
  args: {
    message: "送信エラーが発生しました。時間をおいて再度お試しください。",
    tone: "error",
  },
};
