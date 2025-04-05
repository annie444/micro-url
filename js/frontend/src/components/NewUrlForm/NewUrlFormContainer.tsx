import axios from 'axios'
import { zodResolver } from '@hookform/resolvers/zod'
import { FormProvider, useForm, type SubmitHandler } from 'react-hook-form'

import { Button } from '@/components/ui/button'
import { newUrlFormSchema, type NewUrlFormSchema } from './NewUrlFormSchema'
import { NewUrlForm } from './NewUrlForm'
import { ToastHelper } from '@/helpers/toastHelper'

export function NewUrlFormContainer() {
  const formMethods = useForm<NewUrlFormSchema>({
    resolver: zodResolver(newUrlFormSchema),
    defaultValues: {
      url: '',
      miniUrl: undefined,
    },
    mode: 'onSubmit',
  })

  const onSubmit: SubmitHandler<NewUrlFormSchema> = (values) => {
    const { url, miniUrl } = values
    const response: Promise<{ data: { short_url: string } }> = new Promise(
      (resolve, reject) => {
        axios
          .post('/api/shorten', {
            url: url,
            short: miniUrl,
          })
          .then((res) => {
            resolve(res)
          })
          .catch((err) => {
            reject(err)
          })
      }
    )
    ToastHelper.notifyWithPromise({
      response,
      successMessage: 'URL shortened successfully!',
      errorMessage: 'Error shortening URL',
      successDescription: (res) =>
        'Your shortened URL is: ' + res.data.short_url,
      successAction: (res) => {
        return (
          <Button
            onClick={() => navigator.clipboard.writeText(res.data.short_url)}
          >
            Copy
          </Button>
        )
      },
    })
  }

  return (
    <FormProvider {...formMethods}>
      <NewUrlForm handleSubmit={formMethods.handleSubmit(onSubmit)} />
    </FormProvider>
  )
}
