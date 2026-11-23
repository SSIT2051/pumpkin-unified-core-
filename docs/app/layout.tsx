import { DocsProvider } from 'fumadocs-ui/provider';
import type { RootLayoutProps } from './types';

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="en">
      <body>{<DocsProvider>{children}</DocsProvider>}</body>
    </html>
  );
}
