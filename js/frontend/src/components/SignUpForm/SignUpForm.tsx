import { useFormContext } from "react-hook-form";
import { type SignUpFormSchema } from "./SignUpFormSchema";
import {
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Button } from "@/components/ui/button";
import { Form } from "@/components/shared/Form";
import { EmailField } from "../shared/SignInForm/email_field";
import { PasswordField } from "../shared/SignInForm/password_field";
import { Input } from "../ui/input";

interface SignUpFormProps {
  handleSubmit(): void;
}

export function SignUpForm({ handleSubmit }: SignUpFormProps) {
  const { register } = useFormContext<SignUpFormSchema>();

  return (
    <Form onSubmit={handleSubmit} className="text-center">
      <EmailField className="mb-3" />
      <PasswordField className="mb-3" />
      <FormField
        {...register("name")}
        render={({ field }) => (
          <FormItem>
            <FormLabel>Name</FormLabel>
            <FormControl>
              <Input placeholder="Naomi Campbell" {...field} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
      <Button type="submit">Sign In</Button>
    </Form>
  );
}
