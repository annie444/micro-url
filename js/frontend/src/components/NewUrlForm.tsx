'use client'

import axios from 'axios'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm, type SubmitHandler } from 'react-hook-form'
import { z } from 'zod'
import { toast } from 'sonner'

import { Toaster } from '@/components/ui/sonner'
import { Button } from '@/components/ui/button'
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'

const formSchema = z.object({
  url: z
    .string({
      required_error: 'A URL is required',
      invalid_type_error: 'Must be a valid URL',
    })
    .startsWith('http', {
      message: 'Please use a full URL, including http:// or https://',
    })
    .url({
      message: 'Please enter a valid URL',
    }),
  miniUrl: z.optional(
    z
      .string({
        invalid_type_error: 'Must be a valid url string',
      })
      .min(2, {
        message: 'The micro URL must be at least 2 characters long',
      })
      .max(150, {
        message: 'The micro URL must be at most 150 characters long',
      })
      .regex(/^[A-Za-z0-9-\._~!'()+]+$/, {
        message: 'The micro URL must be a valid url string',
      })
  ),
})

export function NewUrlForm() {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      url: '',
      miniUrl: undefined,
    },
  })

  const onSubmit: SubmitHandler<z.infer<typeof formSchema>> = (
    values: z.infer<typeof formSchema>
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
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
          <FormField
            control={form.control}
            name="url"
            render={({ field }) => (
              <FormItem>
                <FormLabel>URL</FormLabel>
                <FormControl>
                  <Input placeholder="https://google.com/..." {...field} />
                </FormControl>
                <FormDescription>
                  This is the URL you want to shorten.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="miniUrl"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Custom URL</FormLabel>
                <FormControl>
                  <Input placeholder="google" {...field} />
                </FormControl>
                <FormDescription>
                  This is the custom URL you want to use.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <Button type="submit">Submit</Button>
        </form>
      </Form>
      <Toaster />
    </>
  )
}
