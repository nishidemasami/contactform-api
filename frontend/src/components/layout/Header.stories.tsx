import type { Meta, StoryObj } from "@storybook/react";
import { Header } from "./Header";

const meta: Meta<typeof Header> = {
  title: "Layout/Header",
  component: Header,
  tags: ["autodocs"],
};

export default meta;
type Story = StoryObj<typeof Header>;

export const Default: Story = {
  args: {
    title: "サンプルアプリケーション",
  },
};

export const LongTitle: Story = {
  args: {
    title: "長いタイトルのテキストを表示する場合のサンプルアプリケーションヘッダー表示",
  },
};
