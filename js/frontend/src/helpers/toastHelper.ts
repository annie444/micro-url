import type { ReactNode } from 'react'
import { toast, type Action, type ExternalToast } from 'sonner'

const baseOptions: ExternalToast = {
  duration: 4000,
  dismissible: true,
}

type WithPromiseArgs<T> = {
  response: Promise<{ data: T }>
  successMessage: string
  successDescription?: string | ((res: { data: T }) => string)
  successAction?: (res: { data: T }) => ReactNode
  errorMessage: string
}

export class ToastHelper {
  static notify = {
    success(message: string, description?: string) {
      toast.success(message, {
        description,
        ...baseOptions,
        className: 'bg-green-100 text-green-800',
      })
    },
    error(message: string, description?: string) {
      toast.error(message, {
        description,
        ...baseOptions,
        className: 'bg-red-100 text-red-800',
      })
    },
    info(message: string, description?: string) {
      toast.info(message, { description, ...baseOptions })
    },
    global(message: string, description?: string) {
      toast(message, { description, ...baseOptions, position: 'bottom-center' })
    },
  }

  static async notifyWithPromise<T>({
    response,
    successAction,
    successMessage,
    errorMessage,
    successDescription,
  }: WithPromiseArgs<T>) {
    return toast.promise(response, {
      success: (res) => {
        return {
          message: successMessage,
          description: successDescription
            ? typeof successDescription === 'string'
              ? successDescription
              : successDescription(res)
            : undefined,
          closeButton: true,
          dismissible: true,
          duration: 10000,
          action: successAction ? successAction(res) : undefined,
        }
      },
      error: (err) => {
        console.log(err)
        return {
          message: errorMessage,
          closeButton: true,
          dismissible: true,
          duration: 10000,
        }
      },
    })
  }
}
