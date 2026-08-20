import type { Meta, StoryObj } from "@storybook/react";
import { TermsDialog } from "./TermsDialog";

const meta: Meta<typeof TermsDialog> = {
  title: "Inquiry/TermsDialog",
  component: TermsDialog,
  tags: ["autodocs"],
  argTypes: {
    onClose: { action: "closed" },
  },
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
