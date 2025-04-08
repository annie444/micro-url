import { z } from 'zod'

export const signInFormSchema = z.object({
  email: z.string().email(),
  password: z
    .string()
    .min(8)
    .regex(/(?=.*[!@#$%^&*])[a-zA-Z0-9!@#$%^&*]$/, {
      message:
        'Password must contain at least one number and one special character',
    }),
  shouldRemember: z.boolean().optional(),
})

export type SignInFormSchema = z.infer<typeof signInFormSchema>
