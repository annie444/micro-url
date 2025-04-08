import axios from 'axios'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm, FormProvider, type SubmitHandler } from 'react-hook-form'

import { signInFormSchema, type SignInFormSchema } from './SignInFormSchema'
import { SignInForm } from './SignInForm'
import { ToastHelper } from '@/helpers/toastHelper'
import { LocalStorageHelper } from '@/helpers/localStorageHelper'

export function SignInFormContainer() {
  const formMethods = useForm<SignInFormSchema>({
    resolver: zodResolver(signInFormSchema),
    defaultValues: {
      email: '',
      password: '',
    },
  })

  const onSubmit: SubmitHandler<SignInFormSchema> = ({ email, password }) => {
    const response: Promise<{ data: { token: string } }> = new Promise(
      (resolve, reject) => {
        axios
          .post('/auth/login', { email, password })
          .then((res) => {
            if (res.data.token) {
              LocalStorageHelper.setItem('token', res.data.token)
            }
            resolve(res)
          })
          .catch((err) => {
            reject(err)
          })
      }
    )
    ToastHelper.notifyWithPromise({
      response,
      successMessage: 'Logged in successfully!',
      errorMessage: 'Error logging in',
    })
  }

  return (
    <FormProvider {...formMethods}>
      <SignInForm handleSubmit={formMethods.handleSubmit(onSubmit)} />
    </FormProvider>
  )
}
