import { useRoomData } from "./room-provider";
import { CgEnter } from "react-icons/cg";
import { IoMdExit } from "react-icons/io";
import { Box, Flex, Heading, HStack, Textarea, VStack } from "@chakra-ui/react";

export const Chat: React.FC = () => {
  const messages = useRoomData((s) => s.chatMessages);

  return (
    <VStack h="100vh" border="1px" borderColor="gray.700">
      <Flex
        h="3rem"
        alignItems="center"
        borderBottom="1px"
        borderColor="gray.700"
        w="100%"
      >
        <Heading fontSize="2xl" ml="4">
          Chat
        </Heading>
      </Flex>
      <Flex
        overflowY="scroll"
        flexDirection="column"
        w="100%"
        h="calc(100vh - 8rem - 3rem)"
      >
        {messages.map((v, i) => (
          <HStack
            key={i}
            w="100%"
            px="4"
            py="1"
            gap="2"
            _hover={{ bgColor: "gray.800" }}
          >
            {v.t === "Connection" && (
              <>
                <CgEnter size={20} color="#777" />
                {v.c.username} connected
              </>
            )}

            {v.t === "Disconnection" && (
              <>
                <CgEnter size={20} color="#777" />
                {v.c.username} disconnected
              </>
            )}
          </HStack>
        ))}
      </Flex>
      <Box w="100%" h="8rem">
        <Textarea
          h="full"
          resize="none"
          borderRadius="0"
          bg="gray.800"
          onKeyPress={(e) => {
            if (!e.shiftKey && e.key === "Enter") {
              console.log(e.currentTarget.value);
              e.preventDefault();
              e.currentTarget.value = "";
            }
          }}
          placeholder="Chat here.."
        />
      </Box>
    </VStack>
  );
};
