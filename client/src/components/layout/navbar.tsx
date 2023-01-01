import {
  Box,
  Button,
  Center,
  Flex,
  Heading,
  HStack,
  Text,
} from "@chakra-ui/react";
import { signIn, signOut, useSession } from "next-auth/react";

export const Navbar: React.FC = () => {
  const { data: session, status } = useSession();

  return (
    <Center
      position="fixed"
      top="0"
      left="0"
      w="100vw"
      h="14"
      bg="whiteAlpha.50"
    >
      <HStack w="6xl" mx="4">
        <Heading fontSize="lg">Radix</Heading>
        <Heading fontWeight="normal" fontSize="sm">
          a binarysearch clone
        </Heading>

        {status !== "loading" && (
          <HStack ml="auto !important" pr="2">
            {status === "authenticated" ? (
              <>
                <Text>{session.user?.name}</Text>
                <Button colorScheme="red" size="sm" onClick={() => signOut()}>
                  Sign out
                </Button>
              </>
            ) : (
              <Button
                colorScheme="blue"
                size="sm"
                onClick={() => signIn("google")}
              >
                Sign in
              </Button>
            )}
          </HStack>
        )}
      </HStack>
    </Center>
  );
};
