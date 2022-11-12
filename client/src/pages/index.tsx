import { Layout } from "@/components/layout/layout";
import { Navbar } from "@/components/layout/navbar";
import { Box, Button, Heading, Text } from "@chakra-ui/react";
import type { NextPage } from "next";
import { signIn, signOut, useSession } from "next-auth/react";

const Home: NextPage = () => {
  return (
    <Layout title="Radix">
      <Heading>ASdasd</Heading>
    </Layout>
  );
};

export default Home;
