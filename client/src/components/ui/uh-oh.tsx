import {
  Center,
  Box,
  Heading,
  Text,
  Code,
  Button,
  HStack,
} from "@chakra-ui/react";
import { useRouter } from "next/router";

export const UhOh: React.FC<{
  code: "NOT_FOUND" | "UNAUTHORIZED" | "FORBIDDEN" | "INTERNAL_SERVER_ERROR";
  message: string;
}> = ({ code, message }) => {
  const router = useRouter();

  return (
    <Center w="100vw" h="100vh">
      <Box w="36rem" bg="whiteAlpha.100" borderRadius="md" p="4">
        <Heading fontSize="xl">Uh oh!</Heading>

        <hr />

        <Code fontSize="3xl" w="full" textAlign="center">
          {code}
        </Code>

        <Text fontSize="xl" textAlign="center">
          {message}
        </Text>

        <hr />

        <Button onClick={() => router.back()} colorScheme="green">
          Back
        </Button>
        <Button ml="2" onClick={() => router.reload()}>
          Retry
        </Button>
      </Box>
    </Center>
  );
};
