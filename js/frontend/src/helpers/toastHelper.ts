import type { ReactNode } from 'react'
import { toast, type Action, type ExternalToast } from 'sonner'

const baseOptions: ExternalToast = {
  closeButton: true,
  dismissible: true,
  duration: 10000,
}

type NotifyProps = {
  message: string
  description: string
  options?: ExternalToast
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
    success({ message, description, options }: NotifyProps) {
      toast.success(message, {
        description,
        ...baseOptions,
        ...options,
        className: 'bg-green-100 text-green-800',
      })
    },
    error({ message, description, options }: NotifyProps) {
      toast.error(message, {
        description,
        ...baseOptions,
        ...options,
        className: 'bg-red-100 text-red-800',
      })
    },
    info({ message, description, options }: NotifyProps) {
      toast.info(message, {
        description,
        ...baseOptions,
        ...options,
      })
    },
    global({ message, description, options }: NotifyProps) {
      toast(message, {
        description,
        ...baseOptions,
        ...options,
        position: 'bottom-center',
      })
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
          ...baseOptions,
          action: successAction ? successAction(res) : undefined,
        }
      },
      error: (err) => {
        console.log(err)
        return {
          message: errorMessage,
          ...baseOptions,
        }
      },
    })
  }
}
