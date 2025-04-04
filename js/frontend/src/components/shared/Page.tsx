import type { PropsWithChildren } from 'react'
import { Footer } from '@/components/shared/Footer'
import classNames from 'classnames'

type PageProps = PropsWithChildren

export function Page({ children }: PageProps) {
  return (
    <div className={classNames('h-screen', 'grid grid-rows-[1fr_auto]')}>
      <div
        className={classNames(
          'container mx-auto',
          'w-full max-w-4xl',
          'grid content-center',
          'p-4'
        )}
      >
        {children}
      </div>
      <Footer />
    </div>
  )
}
