type PagePath = "index" | "sign_in" | "sign_up";

interface PageConfig {
  title: string;
  href: string;
  description: string;
}

export type PagesConfig = Record<PagePath, PageConfig>;
