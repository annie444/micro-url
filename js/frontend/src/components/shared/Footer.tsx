const currentYear = new Date().getFullYear();

export function Footer() {
  return (
    <footer className="h-fit bg-black py-6 text-white">
      <div className="container mx-auto text-center">
        <p>&copy; {currentYear} All rights reserved.</p>
        <ul className="mt-4 flex justify-center space-x-6">
          <li>
            <a
              href="https://github.com/annie444/micro-url"
              className="hover:text-emerald-400 hover:underline"
            >
              GitHub
            </a>
          </li>
          <li>
            <a
              href="https://github.com/annie444"
              className="hover:text-emerald-400 hover:underline"
            >
              Annie Ehler
            </a>
          </li>
          <li>
            <a
              href="https://github.com/kip-west"
              className="hover:text-emerald-400 hover:underline"
            >
              Kip West
            </a>
          </li>
        </ul>
      </div>
    </footer>
  );
}
