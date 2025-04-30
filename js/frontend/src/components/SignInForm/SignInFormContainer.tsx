import { zodResolver } from "@hookform/resolvers/zod";
import { useForm, FormProvider, type SubmitHandler } from "react-hook-form";

import { signInFormSchema, type SignInFormSchema } from "./SignInFormSchema";
import { SignInForm } from "./SignInForm";
import { ToastHelper, LocalStorageHelper } from "@/helpers";
import { user } from "@/lib/api";

export function SignInFormContainer() {
  const formMethods = useForm<SignInFormSchema>({
    resolver: zodResolver(signInFormSchema),
    defaultValues: {
      email: "",
      password: "",
    },
  });

  const onSubmit: SubmitHandler<SignInFormSchema> = async ({
    email,
    password,
  }) => {
    await ToastHelper.notifyWithPromise({
      response: user.loginLocal({ email, password }),
      successMessage: "Logged in successfully!",
      errorMessage: "Error logging in",
    }).then(async (response) => {
      const user = await response.unwrap();

      LocalStorageHelper.setItem("token", user.user_id);
    });
  };

  return (
    <FormProvider {...formMethods}>
      <SignInForm handleSubmit={formMethods.handleSubmit(onSubmit)} />
    </FormProvider>
  );
}
