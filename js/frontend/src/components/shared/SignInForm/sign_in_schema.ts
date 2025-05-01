import { z } from "zod";

export const emailSchema = z.object({
  email: z.string().trim().email(),
});

export const passwordSchema = z.object({
  password: z
    .string()
    .min(8)
    .regex(/(?=.*[!@#$%^&*])[a-zA-Z0-9!@#$%^&*]$/, {
      message:
        "Password must contain at least one number and one special character",
    }),
});
