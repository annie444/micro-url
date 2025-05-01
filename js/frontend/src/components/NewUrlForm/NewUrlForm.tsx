"use client";

import { Button } from "@/components/ui/button";
import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { useFormContext } from "react-hook-form";
import { type NewUrlFormSchema } from "./NewUrlFormSchema";
import { Form } from "@/components/shared/Form";

interface NewUrlFormProps {
  handleSubmit(): void;
}

export function NewUrlForm({ handleSubmit }: NewUrlFormProps) {
  const { register } = useFormContext<NewUrlFormSchema>();

  return (
    <Form onSubmit={handleSubmit}>
      <FormField
        {...register("url")}
        render={({ field }) => (
          <FormItem>
            <FormLabel>URL</FormLabel>
            <FormControl>
              <Textarea placeholder="https://google.com/..." {...field} />
            </FormControl>
            <FormDescription>
              This is the URL you want to shorten.
            </FormDescription>
            <FormMessage />
          </FormItem>
        )}
      />
      <FormField
        {...register("short")}
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
    </Form>
  );
}
