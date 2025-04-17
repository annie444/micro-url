import { zodResolver } from "@hookform/resolvers/zod";
import { FormProvider, useForm, type SubmitHandler } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { newUrlFormSchema, type NewUrlFormSchema } from "./NewUrlFormSchema";
import { NewUrlForm } from "./NewUrlForm";
import { ToastHelper, LocalStorageHelper } from "@/helpers";
import { useEffect } from "react";
import { urls } from "@/lib/api";
import type { NewUrlRequest, ShortLink } from "@/lib/types";

export function NewUrlFormContainer() {
  const formMethods = useForm<NewUrlFormSchema>({
    resolver: zodResolver(newUrlFormSchema),
    defaultValues: {
      url: "",
      short: undefined,
    },
    mode: "onSubmit",
  });

  const onSubmit: SubmitHandler<NewUrlFormSchema> = (values) => {
    const { url, short } = values;
    const requestParams: NewUrlRequest = {
      url: url,
      short: short,
    };
    ToastHelper.notifyWithPromise({
      response: urls.newUrl(requestParams),
      successMessage: "URL shortened successfully!",
      errorMessage: "Error shortening URL",
      successDescription: (res: ShortLink) =>
        "Your shortened URL is: " + res.short_url,
      successAction: (res: ShortLink) => {
        return (
          <Button onClick={() => navigator.clipboard.writeText(res.short_url)}>
            Copy
          </Button>
        );
      },
    });
  };

  useEffect(() => {
    if (!LocalStorageHelper.getItem("hasDismissedWelcomeToast")) {
      ToastHelper.notify.global({
        message: "Welcome to MicroUrl!",
        description:
          "We use cookies for authentication purposes which fall under the realm of “necessary functionality” and therefore can't be removed.",
        options: {
          onDismiss: () => {
            LocalStorageHelper.setItem("hasDismissedWelcomeToast", "true");
          },
        },
      });
    }
  }, []);

  return (
    <FormProvider {...formMethods}>
      <NewUrlForm handleSubmit={formMethods.handleSubmit(onSubmit)} />
    </FormProvider>
  );
}
