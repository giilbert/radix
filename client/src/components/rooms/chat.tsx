import { useRoom, useRoomData } from "./room-provider";
import { CgEnter } from "react-icons/cg";
import { TbConfetti, TbTrophy } from "react-icons/tb";
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
import { createRef, useEffect, useRef } from "react";
import { FiFlag, FiUpload } from "react-icons/fi";
import { MarkdownRender } from "../ui/markdown-render";

const nth = (n: number) => {
  return ["st", "nd", "rd"][((((n + 90) % 100) - 10) % 10) - 1] || "th";
};

export const Chat: React.FC = () => {
  const room = useRoom();
  const messages = useRoomData((s) => s.chatMessages);
  const users = useRoomData((s) => s.users);
  const containerRef = createRef<HTMLDivElement>();
  const hasInit = useRef(false);

  useEffect(() => {
    if (containerRef.current) {
      const el = containerRef.current;
      const maxScroll = el.scrollHeight - el.clientHeight;

      const maxScrollBefore =
        maxScroll - ((el.lastChild as HTMLElement | null)?.clientHeight || 0);

      if (Math.abs(maxScrollBefore - el.scrollTop) < 30) {
        el.scrollTop = maxScroll;
      }
    }
  }, [messages.length, containerRef]);

  useEffect(() => {
    if (containerRef.current && messages.length !== 0 && !hasInit.current) {
      const maxScroll =
        containerRef.current.scrollHeight - containerRef.current.clientHeight;
      containerRef.current.scrollTop = maxScroll;

      console.log("scrolling");
      hasInit.current = true;
    }
  }, [containerRef, messages.length]);

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
                alt={""}
                bg="white"
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
        ref={containerRef}
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
            {v.t === "RoundBegin" && (
              <>
                <FiFlag size={20} color="#48BB78" />
                Round began!
              </>
            )}

            {v.t === "UserSubmitted" && (
              <>
                <FiUpload size={20} color="#777" />
                {v.c.username} submitted
              </>
            )}

            {v.t === "UserProblemCompletion" && (
              <>
                <TbConfetti size={20} color="#48BB78" />
                {v.c.username} completed problem {v.c.problemIndex + 1}!
              </>
            )}

            {v.t === "UserFinished" && (
              <>
                <TbTrophy size={20} color="#F4D03F" />
                {v.c.username} completed in {v.c.place}
                {nth(v.c.place)} place!
              </>
            )}

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
                    wordBreak="break-all"
                  >
                    {v.c.author.name}
                  </Text>

                  <MarkdownRender content={v.c.content} />
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
            if (e.key === "Enter" && e.currentTarget.value === "") {
              e.preventDefault();
              return;
            }

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
