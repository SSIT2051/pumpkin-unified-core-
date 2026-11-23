import type { Metadata } from 'next';
import { createMDXSource } from 'fumadocs-mdx';
import { docs } from '@/source.config';
import { NotFound } from 'fumadocs-ui/not-found';
import { Sidebar } from 'fumadocs-ui/sidebar';
import { MobileSidebar } from 'fumadocs-ui/mobile-sidebar';
import { MobileTopbar } from 'fumadocs-ui/mobile-topbar';
import { useSearchParams } from 'next/navigation';

export const source = docs;

export const metadata: Metadata = {
  title: 'Pumpkin Documentation',
  description: 'Documentation for the Pumpkin Minecraft Server',
};

export default function Layout({
  children,
}: {
  children: React.ReactNode;
}) {
  const searchParams = useSearchParams();
  const id = searchParams.get('id');

  return (
    <>
      <MobileTopbar />
      <MobileSidebar />
      <div className="flex flex-col lg:min-h-[calc(100svh-var(--mobile-topbar-height))]">
        <div className="relative flex flex-col min-h-full lg:flex-row lg:min-h-screen">
          <div className="hidden lg:flex lg:w-[var(--sidebar-width)]">
            <Sidebar />
          </div>
          <div className="flex-1 min-w-0">
            <main className="prose prose-invert md:prose-xl max-w-none">
              {children}
            </main>
          </div>
        </div>
      </div>
      <NotFound id={id} />
    </>
  );
}
