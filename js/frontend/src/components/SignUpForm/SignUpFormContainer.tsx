import { useForm, FormProvider, type SubmitHandler } from "react-hook-form";
import { signUpFormSchema, type SignUpFormSchema } from "./SignUpFormSchema";
import { SignUpForm } from "./SignUpForm";
import { zodResolver } from "@hookform/resolvers/zod";
import { user } from "@/lib/api";
import { ToastHelper } from "@/helpers";

export function SignUpFormContainer() {
  const formMethods = useForm<SignUpFormSchema>({
    resolver: zodResolver(signUpFormSchema),
  });

  const onSubmit: SubmitHandler<SignUpFormSchema> = async (values) => {
    await ToastHelper.notifyWithPromise({
      response: user.registerUser(values),
      successMessage: "Signed up successfully!",
      errorMessage: "Error signing up",
    });
  };

  return (
    <FormProvider {...formMethods}>
      <SignUpForm handleSubmit={formMethods.handleSubmit(onSubmit)} />
    </FormProvider>
  );
}
