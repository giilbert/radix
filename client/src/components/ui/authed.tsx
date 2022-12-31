import { Box, Button, Center, Heading, Text } from "@chakra-ui/react";
import { signIn, useSession } from "next-auth/react";
import { ReactNode } from "react";

export const Authed: React.FC<
  React.PropsWithChildren<{
    fallback?: ReactNode;
  }>
> = ({ fallback, children }) => {
  const { status } = useSession();

  if (status === "loading") return <>{fallback}</>;

  if (status === "unauthenticated")
    return (
      <Center w="100vw" h="100vh">
        <Box bg="whiteAlpha.100" p="8" borderRadius="md">
          <Heading>Sign in to access this page</Heading>
          <Button onClick={() => signIn("google")} mt="2">
            Sign in with Google
          </Button>
        </Box>
      </Center>
    );

  return <>{children}</>;
};
