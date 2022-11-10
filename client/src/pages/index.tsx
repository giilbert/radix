import { Box, Button, Text } from "@chakra-ui/react";
import type { NextPage } from "next";
import { signIn, signOut, useSession } from "next-auth/react";

const Home: NextPage = () => {
  const { data: session, status } = useSession();

  if (status === "loading") return <p>Loading</p>;
  if (status === "authenticated")
    return (
      <Box>
        <Text>Signed in as {session.user?.name}</Text>
        <Button colorScheme="red" onClick={() => signOut()}>
          Sign out
        </Button>
      </Box>
    );

  return (
    <Button colorScheme="blue" onClick={() => signIn("google")}>
      Sign in
    </Button>
  );
};

export default Home;
