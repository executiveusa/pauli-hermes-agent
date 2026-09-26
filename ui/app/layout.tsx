import type { Metadata } from "next";
import "./globals.css";
import Nav from "@/components/Nav";

export const metadata: Metadata = {
  title: "Hermes — AI Voice Agent",
  description: "Hermes AI agent — voice, chat, skills, and free NVIDIA inference",
};

export const viewport = {
  width: "device-width",
  initialScale: 1,
  maximumScale: 1,
  themeColor: "#0d0d14",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" style={{ height: '100%', background: '#0d0d14' }}>
      <body style={{ minHeight: '100%', display: 'flex', flexDirection: 'column', margin: 0 }}>
        <Nav />
        <main style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          {children}
        </main>
      </body>
    </html>
  );
}
