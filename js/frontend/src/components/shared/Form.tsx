import { type PropsWithChildren } from "react";
import { cn } from "@/lib/utils";

export function Form({
  children,
  onSubmit,
  className,
}: PropsWithChildren<{ onSubmit(): void; className?: string }>) {
  return (
    <form className={cn("w-full space-y-8", className)} onSubmit={onSubmit}>
      {children}
    </form>
  );
}
