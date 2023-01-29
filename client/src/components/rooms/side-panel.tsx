import { Box, Button, Code, Heading, HStack, Text } from "@chakra-ui/react";
import { useSession } from "next-auth/react";
import { useRouter } from "next/router";
import {
  FiArrowLeft,
  FiArrowRight,
  FiChevronLeft,
  FiChevronRight,
} from "react-icons/fi";
import { MarkdownRender } from "../ui/markdown-render";
import { useRoom, useRoomData } from "./room-provider";

export const SidePanel: React.FC = () => {
  const router = useRouter();
  const { data: session } = useSession();
  const {
    problems,
    roomConfig,
    setCurrentProblemIndex,
    currentProblemIndex,
    setTestStatus,
  } = useRoomData();
  const room = useRoom();

  if (!session || !roomConfig) return <Text>Loading</Text>;

  return (
    <Box m="4">
      <Heading fontSize="2xl">
        <Code fontSize="2xl">
          {router.query.name}
        </Code>
      </Heading>

      <hr />

      {roomConfig.owner.id === session.user.id && !problems && (
        <Button
          bgColor="green.500"
          w="100%"
          mt="4"
          onClick={() => {
            room.sendCommand({
              t: "BeginRound",
              c: null,
            });
          }}
        >
          Start!
        </Button>
      )}

      {problems && (
        <>
          <HStack>
            <Heading fontSize="3xl">
              {problems[currentProblemIndex].title}
            </Heading>

            <HStack ml="auto !important" gap="1">
              <Button
                onClick={() => {
                  setTestStatus({
                    t: "None",
                    c: null,
                  });
                  if (currentProblemIndex !== 0)
                    setCurrentProblemIndex(currentProblemIndex - 1);
                }}
                size="sm"
                px="2"
              >
                <FiChevronLeft />
              </Button>

              <Text>
                {currentProblemIndex + 1} / {problems?.length || 0}
              </Text>

              <Button
                onClick={() => {
                  setTestStatus({
                    t: "None",
                    c: null,
                  });
                  if (currentProblemIndex !== (problems?.length || 0) - 1)
                    setCurrentProblemIndex(currentProblemIndex + 1);
                }}
                size="sm"
                px="2"
              >
                <FiChevronRight />
              </Button>
            </HStack>
          </HStack>
          <MarkdownRender content={problems[currentProblemIndex].description} />
        </>
      )}
    </Box>
  );
};
