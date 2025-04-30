type PagePath = "index" | "sign_in";

interface PageConfig {
  title: string;
  href: string;
  description: string;
}

export type PagesConfig = Record<PagePath, PageConfig>;
