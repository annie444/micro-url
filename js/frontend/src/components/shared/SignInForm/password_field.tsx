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

interface PasswordFieldProps {
  className?: string;
}

export function PasswordField({ className }: PasswordFieldProps) {
  const { register } = useFormContext<SignInFormSchema | SignUpFormSchema>();

  return (
    <FormField
      {...register("password")}
      render={({ field }) => (
        <FormItem className={className}>
          <FormLabel>Password</FormLabel>
          <FormControl>
            <Input placeholder="Abc123!" {...field} />
          </FormControl>
          <FormMessage />
        </FormItem>
      )}
    />
  );
}
