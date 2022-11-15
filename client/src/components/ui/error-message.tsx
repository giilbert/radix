import { Box, HStack, Code } from "@chakra-ui/react";
import { AxiosError } from "axios";
import { FiX } from "react-icons/fi";

export const ErrorMessage: React.FC<{
  error?: AxiosError | null;
}> = ({ error }) => {
  if (!error) return null;

  return (
    <HStack bgColor="red.500" rounded="md" p="2">
      <FiX size={24} />
      <Box fontSize="lg">
        Error: <Code fontSize="lg">{error.code?.toUpperCase()}</Code>
        {error.message && ":"} {error.message}
      </Box>
    </HStack>
  );
};
