import { Box, Center } from "@chakra-ui/react";
import Head from "next/head";
import { Navbar } from "./navbar";

export const Layout: React.FC<
  React.PropsWithChildren<{
    title: string;
  }>
> = ({ title, children }) => (
  <>
    <Head>
      <title>{title}</title>
    </Head>

    <Navbar />

    <Center mt="20" mx="4">
      <Box w="6xl">{children}</Box>
    </Center>
  </>
);
