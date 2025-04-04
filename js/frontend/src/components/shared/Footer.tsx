const currentYear = new Date().getFullYear()

export function Footer() {
  return (
    <footer className="bg-black text-white py-6 h-fit">
      <div className="container mx-auto text-center">
        <p>&copy; {currentYear} All rights reserved.</p>
        <ul className="flex justify-center space-x-6 mt-4">
          <li>
            <a
              href="https://github.com/annie444/micro-url"
              className="hover:underline hover:text-emerald-400"
            >
              GitHub
            </a>
          </li>
          <li>
            <a
              href="https://github.com/annie444"
              className="hover:underline hover:text-emerald-400"
            >
              Annie Ehler
            </a>
          </li>
          <li>
            <a
              href="https://github.com/kip-west"
              className="hover:underline hover:text-emerald-400"
            >
              Kip West
            </a>
          </li>
        </ul>
      </div>
    </footer>
  )
}
