import { Box, Center } from "@chakra-ui/react";
import Head from "next/head";
import { Navbar } from "./navbar";

export const Layout: React.FC<
  React.PropsWithChildren<{
    title: string;
    selectedPage?: "rooms" | "problems";
  }>
> = ({ title, children, selectedPage }) => (
  <>
    <Head>
      <title>{title + " | Radix"}</title>
    </Head>

    <Navbar selectedPage={selectedPage} />

    <Center mt="20" mx="4">
      <Box w="6xl">{children}</Box>
    </Center>
  </>
);
