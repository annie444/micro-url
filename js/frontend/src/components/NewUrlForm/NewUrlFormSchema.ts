import { z } from "zod";

export const newUrlFormSchema = z.object({
  url: z
    .string({
      required_error: "A URL is required",
      invalid_type_error: "Must be a valid URL",
    })
    .startsWith("http", {
      message: "Please use a full URL, including http:// or https://",
    })
    .url({
      message: "Please enter a valid URL",
    }),
  miniUrl: z.optional(
    z
      .string({
        invalid_type_error: "Must be a valid url string",
      })
      .min(2, {
        message: "The micro URL must be at least 2 characters long",
      })
      .max(150, {
        message: "The micro URL must be at most 150 characters long",
      })
      .regex(/^[A-Za-z0-9-._~!'()+]+$/, {
        message: "The micro URL must be a valid url string",
      }),
  ),
});

export type NewUrlFormSchema = z.infer<typeof newUrlFormSchema>;
