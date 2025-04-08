import { useFormContext } from 'react-hook-form'
import { type SignInFormSchema } from './SignInFormSchema'
import {
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Form } from '@/components/shared/Form'
import { Checkbox } from '@/components/ui/checkbox'

type SignInFormProps = {
  handleSubmit(): void
}

export function SignInForm({ handleSubmit }: SignInFormProps) {
  const { register } = useFormContext<SignInFormSchema>()

  return (
    <Form onSubmit={handleSubmit} className="text-center">
      <div className="mb-4">
        <h1 className="text-2xl font-bold text-center">MicroURL</h1>
        <p className="text-center text-sm text-gray-500">
          Welcome to MicroURL!
        </p>
      </div>
      <FormField
        {...register('email')}
        render={({ field }) => (
          <FormItem>
            <FormLabel>Email</FormLabel>
            <FormControl>
              <Input placeholder="email@example.com" {...field} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
      <FormField
        {...register('password')}
        render={({ field }) => (
          <FormItem className="mb-4">
            <FormLabel>Password</FormLabel>
            <FormControl>
              <Input placeholder="Abc123!" {...field} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
      <div className="flex justify-between mb-2">
        <FormField
          {...register('shouldRemember')}
          render={({ field }) => (
            <FormItem className="flex items-center gap-2">
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
  )
}
