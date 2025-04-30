import type { PagesConfig } from "@/types/pages";

export const BASE_PATH = "/ui";

export const PAGES_CONFIG: PagesConfig = {
  index: {
    title: "Home",
    href: `${BASE_PATH}/`,
    description: "Home",
  },
  sign_in: {
    title: "Sign In",
    href: `${BASE_PATH}/sign_in`,
    description: "Sign In",
  },
};
