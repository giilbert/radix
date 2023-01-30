import { Button, Center, Heading, HStack, Text } from "@chakra-ui/react";
import { signIn, signOut, useSession } from "next-auth/react";
import Link from "next/link";

export const Navbar: React.FC<{ selectedPage?: "rooms" | "problems" }> = ({
  selectedPage,
}) => {
  const { data: session, status } = useSession();

  return (
    <Center
      position="fixed"
      top="0"
      left="0"
      w="100vw"
      h="14"
      bg="#232934"
      zIndex="99"
    >
      <HStack w="6xl" mx="4" gap="1">
        <Heading fontSize="xl" mr="4">
          Radix
        </Heading>

        <Link href="/">
          <Text
            fontWeight={selectedPage === "rooms" ? "extrabold" : "normal"}
            _hover={{
              textDecoration: "underline",
            }}
          >
            Rooms
          </Text>
        </Link>
        <Link href="/problems">
          <Text
            fontWeight={selectedPage === "problems" ? "extrabold" : "normal"}
            _hover={{
              textDecoration: "underline",
            }}
          >
            Problems
          </Text>
        </Link>

        {status !== "loading" && (
          <HStack ml="auto !important" pr="2">
            {status === "authenticated" ? (
              <>
                <Text>{session.user?.name}</Text>
                <Button
                  bgColor="red.500"
                  _hover={{ bgColor: "red.400" }}
                  _active={{ bgColor: "red.500" }}
                  size="sm"
                  onClick={() => signOut()}
                >
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
