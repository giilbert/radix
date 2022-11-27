import { Box, Heading } from "@chakra-ui/react";
import { useRouter } from "next/router";

export const SidePanel: React.FC = () => {
  const router = useRouter();

  return (
    <Box m="4">
      <Heading fontSize="2xl">{router.query.name}</Heading>
    </Box>
  );
};
