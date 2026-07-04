import "./globals.css";
import {Metadata} from "next";
import {PorscheNext, PlexSans, PlexMono} from "@/lib/fonts";
import clsx from "clsx";
import Script from "next/script";
import React from "react";

export const metadata: Metadata = {
  title: "Libretto",
  description: "Technical service reference for exotic vehicles",
};

export default function RootLayout({
                                     children,
                                   }: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${PorscheNext.variable} ${PlexSans.variable} ${PlexMono.variable}`}>
    <body className={clsx(`font-sans text-ink bg-paper antialiased`)}>
    {children}
    </body>
    </html>
  );
}
