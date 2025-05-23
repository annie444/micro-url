import { z } from "zod";
import {
  emailSchema,
  passwordSchema,
} from "../shared/SignInForm/sign_in_schema";

export const signInFormSchema = z
  .object({
    shouldRemember: z.boolean().optional(),
  })
  .merge(emailSchema)
  .merge(passwordSchema);

export type SignInFormSchema = z.infer<typeof signInFormSchema>;
