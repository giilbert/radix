import { useRoom, useRoomData } from "./room-provider";
import { CgEnter } from "react-icons/cg";
import { IoMdExit } from "react-icons/io";
import {
  Box,
  Flex,
  Heading,
  HStack,
  Image,
  Text,
  Textarea,
  Tooltip,
  VStack,
} from "@chakra-ui/react";

export const Chat: React.FC = () => {
  const room = useRoom();
  const messages = useRoomData((s) => s.chatMessages);
  const users = useRoomData((s) => s.users);

  return (
    <VStack h="100vh" border="1px" borderColor="gray.700">
      <Box h="4rem" borderBottom="1px" borderColor="gray.700" w="100%" mt="4">
        <HStack mt="2" ml="4">
          {users.map((user) => (
            <Tooltip key={user.id} label={user.name}>
              <Image
                src={user.image}
                w="32px"
                borderRadius={999}
                alt=""
                cursor="pointer"
              />
            </Tooltip>
          ))}
        </HStack>
      </Box>
      <Flex
        overflowY="scroll"
        flexDirection="column"
        w="100%"
        h="calc(100vh - 8rem - 4rem)"
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

            {v.t === "UserChat" && (
              <HStack gap="1">
                <Image
                  src={v.c.author.image}
                  w="24px"
                  borderRadius={999}
                  alt=""
                  cursor="pointer"
                />
                <Box pt="3">
                  <Text
                    color="gray.100"
                    _hover={{
                      cursor: "pointer",
                      textDecoration: "underline",
                    }}
                  >
                    {v.c.author.name}
                  </Text>
                  {v.c.content.split("\n").map((line, i) => (
                    <Text color="gray.400" key={i}>
                      {line || " "}
                    </Text>
                  ))}
                </Box>
              </HStack>
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
              room.sendCommand({
                t: "SendChatMessage",
                c: {
                  content: e.currentTarget.value,
                },
              });
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
