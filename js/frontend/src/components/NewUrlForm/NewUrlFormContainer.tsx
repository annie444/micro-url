'use client'

import axios from 'axios'
import { zodResolver } from '@hookform/resolvers/zod'
import { FormProvider, useForm, type SubmitHandler } from 'react-hook-form'
import { toast } from 'sonner'

import { Toaster } from '@/components/ui/sonner'
import { Button } from '@/components/ui/button'
import { newUrlFormSchema, type NewUrlFormSchema } from './NewUrlFormSchema'
import { NewUrlForm } from './NewUrlForm'

export function NewUrlFormContainer() {
  const formMethods = useForm<NewUrlFormSchema>({
    resolver: zodResolver(newUrlFormSchema),
    defaultValues: {
      url: '',
      miniUrl: undefined,
    },
  })

  const onSubmit: SubmitHandler<NewUrlFormSchema> = (
    values: NewUrlFormSchema
  ) => {
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
    toast.promise(response, {
      success: (res) => {
        return {
          message: 'URL shortened successfully!',
          description: 'Your shortened URL is: ' + res.data.short_url,
          closeButton: true,
          dismissible: true,
          duration: 10000,
          action: (
            <Button
              onClick={() => navigator.clipboard.writeText(res.data.short_url)}
            >
              Copy
            </Button>
          ),
        }
      },
      error: (err) => {
        console.log(err)
        return {
          message: 'Error shortening URL',
          closeButton: true,
          dismissible: true,
          duration: 10000,
        }
      },
    })
  }

  return (
    <>
      <FormProvider {...formMethods}>
        <NewUrlForm handleSubmit={formMethods.handleSubmit(onSubmit)} />
      </FormProvider>
      <Toaster />
    </>
  )
}
