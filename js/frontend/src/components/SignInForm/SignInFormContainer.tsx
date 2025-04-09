import axios, { type AxiosResponse } from "axios";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm, FormProvider, type SubmitHandler } from "react-hook-form";

import { signInFormSchema, type SignInFormSchema } from "./SignInFormSchema";
import { SignInForm } from "./SignInForm";
import { ToastHelper, LocalStorageHelper } from "@/helpers";
import type { User } from "@/lib/types/User";

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
    const response = new Promise<AxiosResponse<User>>((resolve, reject) => {
      axios
        .post("/auth/login", { email, password })
        .then((res) => {
          resolve(res);
        })
        .catch((err) => {
          reject(err);
        });
    });
    await ToastHelper.notifyWithPromise({
      response,
      successMessage: "Logged in successfully!",
      errorMessage: "Error logging in",
    });
    LocalStorageHelper.setItem("token", (await response).data.user_id);
  };

  return (
    <FormProvider {...formMethods}>
      <SignInForm handleSubmit={formMethods.handleSubmit(onSubmit)} />
    </FormProvider>
  );
}
