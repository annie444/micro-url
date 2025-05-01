import { z } from "zod";
import {
  emailSchema,
  passwordSchema,
} from "../shared/SignInForm/sign_in_schema";

export const signUpFormSchema = z
  .object({
    name: z.string().trim().min(1),
  })
  .merge(emailSchema)
  .merge(passwordSchema);

export type SignUpFormSchema = z.infer<typeof signUpFormSchema>;
