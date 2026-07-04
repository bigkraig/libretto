import "./globals.css";
import {Metadata} from "next";
import {PorscheNext} from "@/lib/fonts";
import clsx from "clsx";
import Script from "next/script";
import React from "react";

export const metadata: Metadata = {
  title: "Libretto",
  description: "Technical Service Information",
};

export default function RootLayout({
                                     children,
                                   }: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${PorscheNext.variable}`}>
    <body className={clsx(`font-medium bg-white`)}>
    {children}
    </body>
    </html>
  );
}
