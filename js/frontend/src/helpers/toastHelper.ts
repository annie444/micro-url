import type { ReactNode } from "react";
import { toast, type ExternalToast } from "sonner";

const baseOptions: ExternalToast = {
  closeButton: true,
  dismissible: true,
  duration: 10000,
};

interface NotifyProps {
  message: string;
  description: string;
  options?: ExternalToast;
}

interface WithPromiseArgs<T> {
  response: Promise<T>;
  successMessage: string;
  successDescription?: string | ((res: T) => string);
  successAction?: (res: T) => ReactNode;
  errorMessage: string;
}

export const notify = {
  success({ message, description, options }: NotifyProps) {
    toast.success(message, {
      description,
      ...baseOptions,
      ...options,
      className: "bg-green-100 text-green-800",
    });
  },
  error({ message, description, options }: NotifyProps) {
    toast.error(message, {
      description,
      ...baseOptions,
      ...options,
      className: "bg-red-100 text-red-800",
    });
  },
  info({ message, description, options }: NotifyProps) {
    toast.info(message, {
      description,
      ...baseOptions,
      ...options,
    });
  },
  global({ message, description, options }: NotifyProps) {
    toast(message, {
      description,
      ...baseOptions,
      ...options,
      duration: Infinity,
      position: "bottom-center",
    });
  },
};

export async function notifyWithPromise<T>({
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
          ? typeof successDescription === "string"
            ? successDescription
            : successDescription(res)
          : undefined,
        ...baseOptions,
        action: successAction ? successAction(res) : undefined,
      };
    },
    error: (err) => {
      console.log(err);
      return {
        message: errorMessage,
        ...baseOptions,
      };
    },
  });
}
