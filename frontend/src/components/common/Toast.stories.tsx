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
