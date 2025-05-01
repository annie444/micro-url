import type { SignInFormSchema } from "@/components/SignInForm/SignInFormSchema";
import type { SignUpFormSchema } from "@/components/SignUpForm/SignUpFormSchema";
import {
  FormField,
  FormLabel,
  FormControl,
  FormMessage,
  FormItem,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { useFormContext } from "react-hook-form";

interface EmailFieldProps {
  className?: string;
}

export function EmailField({ className }: EmailFieldProps) {
  const { register } = useFormContext<SignInFormSchema | SignUpFormSchema>();

  return (
    <FormField
      {...register("email")}
      render={({ field }) => (
        <FormItem className={className}>
          <FormLabel>Email</FormLabel>
          <FormControl>
            <Input placeholder="email@example.com" {...field} />
          </FormControl>
          <FormMessage />
        </FormItem>
      )}
    />
  );
}
