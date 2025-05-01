import { useFormContext } from "react-hook-form";
import { type SignInFormSchema } from "./SignInFormSchema";
import {
  FormControl,
  FormField,
  FormItem,
  FormLabel,
} from "@/components/ui/form";
import { Button } from "@/components/ui/button";
import { Form } from "@/components/shared/Form";
import { Checkbox } from "@/components/ui/checkbox";
import { EmailField } from "../shared/SignInForm/email_field";
import { PasswordField } from "../shared/SignInForm/password_field";

interface SignInFormProps {
  handleSubmit(): void;
}

export function SignInForm({ handleSubmit }: SignInFormProps) {
  const { register } = useFormContext<SignInFormSchema>();

  return (
    <Form onSubmit={handleSubmit} className="text-center">
      <EmailField className="mb-3" />
      <PasswordField className="mb-3" />
      <div className="mb-4 flex justify-between">
        <FormField
          {...register("shouldRemember")}
          render={({ field }) => (
            <FormItem className="flex items-center">
              <FormLabel>Remember me</FormLabel>
              <FormControl>
                <Checkbox {...field} />
              </FormControl>
            </FormItem>
          )}
        />
        <Button variant="link" className="p-0">
          Forgot Password?
        </Button>
      </div>
      <Button type="submit">Sign In</Button>
    </Form>
  );
}
