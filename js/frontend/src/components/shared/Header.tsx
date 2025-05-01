import { cn } from "@/lib/utils";
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from "@/components/ui/navigation-menu";
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { PAGES_CONFIG } from "@/constants/pages";
import type { PagesConfig } from "@/types/pages";

export function Header() {
  return (
    <header className={"p-3"}>
      <div className="container mx-auto">
        <NavigationMenu className="relative flex max-w-full justify-center md:justify-between">
          <NavigationMenuLink
            href={PAGES_CONFIG.index.href}
            className={cn("text-xl font-bold")}
          >
            <h1>MicroUrl</h1>
          </NavigationMenuLink>
          <Sheet>
            <SheetTrigger asChild>
              <Button variant="ghost" className="absolute right-0 md:hidden">
                ☰<span className="sr-only">Toggle navigation menu</span>
              </Button>
            </SheetTrigger>
            <SheetContent side="top">
              <PagesLinks pagesConfig={PAGES_CONFIG} />
            </SheetContent>
          </Sheet>
          <div className={cn("hidden md:flex")}>
            <PagesLinks pagesConfig={PAGES_CONFIG} />
          </div>
        </NavigationMenu>
      </div>
    </header>
  );
}

interface PagesLinksProps {
  pagesConfig: PagesConfig;
}

function PagesLinks({ pagesConfig }: PagesLinksProps) {
  return (
    <NavigationMenuList>
      {Object.entries(pagesConfig)
        .filter(([key]) => key !== "index")
        .map(([key, value]) => (
          <NavigationMenuItem key={key}>
            <NavigationMenuLink href={value.href}>
              {value.title}
            </NavigationMenuLink>
          </NavigationMenuItem>
        ))}
    </NavigationMenuList>
  );
}
