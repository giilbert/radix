import ReactCodeMirror from "@uiw/react-codemirror";
import { langs } from "@uiw/codemirror-extensions-langs";
import { githubDark } from "@uiw/codemirror-themes-all";
import {
  Box,
  Button,
  HStack,
  Select,
  Tab,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
  Text,
} from "@chakra-ui/react";
import { useRoom, useRoomData } from "./room-provider";
import { useSession } from "next-auth/react";
import { throttle } from "@/lib/utils/throttle";
import { useState } from "react";
import { TestResults } from "./test-results";
import { useIsMobile } from "@/lib/hooks/use-is-mobile";

export const Editors: React.FC = () => {
  const { data: session } = useSession();
  const room = useRoom();
  const {
    users: usersUnsorted,
    problems,
    code,
    currentProblemIndex,
    setProblemCode,
    testStatus,
    setTestStatus,
  } = useRoomData();
  const [selectedLanguage, setSelectedLanguage] = useState<
    "python" | "javascript"
  >("python");
  const [currentTab, setCurrentTab] = useState(0);
  const isMobile = useIsMobile();

  const isResultsActive = testStatus.t !== "None";

  const currentUserIndex = usersUnsorted.findIndex(
    (user) => user.id === session?.user.id
  );
  const users = [...usersUnsorted];
  users.splice(currentUserIndex, 1);
  if (usersUnsorted[currentUserIndex])
    users.unshift(usersUnsorted[currentUserIndex]);

  if (users.length === 0) return <Text>Loading</Text>;
  if (!session) return <Text>Loading</Text>;

  return (
    <Box>
      <Tabs
        h="100%"
        defaultIndex={users.findIndex((user) => user.id === session.user.id)}
        index={currentTab}
        onChange={(e) => {
          setCurrentTab(e);
        }}
      >
        <TabList h="3rem">
          {users.map((user) => (
            <Tab key={user.id}>
              {user.id === session.user.id ? "You" : user.name}
            </Tab>
          ))}

          {isMobile && !problems && (
            <Button
              ml="auto !important"
              m="2"
              size="sm"
              w="20"
              bgColor="green.500"
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
        </TabList>

        <TabPanels
          h={`calc(100vh${
            isResultsActive
              ? isMobile
                ? " - 29rem"
                : " - 26rem"
              : isMobile
              ? " - 9rem"
              : " - 6rem"
          })`}
        >
          {users.map((user) => {
            return (
              <TabPanel p="0" h="100%" key={user.id}>
                {user.id !== session.user.id && (
                  <Text key={user.id} p="4" fontSize="xl">
                    TODO: see other people&apos;s editor
                  </Text>
                )}

                {user.id === session.user.id && (
                  <ReactCodeMirror
                    theme={[githubDark]}
                    extensions={[langs[selectedLanguage]()]}
                    lang={selectedLanguage}
                    editable={!!problems}
                    placeholder={
                      !problems ? "Waiting for the round to begin..." : ""
                    }
                    value={code.get(currentProblemIndex)?.get(selectedLanguage)}
                    onChange={throttle((content: string) => {
                      setProblemCode(
                        currentProblemIndex,
                        selectedLanguage,
                        content
                      );
                      if (problems)
                        room.sendCommand({
                          t: "SetEditorContent",
                          c: {
                            content,
                          },
                        });
                    })}
                  />
                )}
              </TabPanel>
            );
          })}
        </TabPanels>
      </Tabs>

      <Box>
        {isResultsActive && <TestResults />}

        <HStack alignItems="center" h="3rem" justify="flex-end" pr="1">
          <Select
            mr="auto"
            w="36"
            onChange={(event) => {
              setSelectedLanguage(
                event.currentTarget.value.toLowerCase() as any
              );
            }}
          >
            <option>Python</option>
            {/* <option>JavaScript</option> */}
          </Select>

          <Button
            w="28"
            isLoading={testStatus.t === "Awaiting"}
            onClick={throttle(() => {
              if (!problems) return;

              room.sendCommand({
                t: "SetEditorContent",
                c: {
                  content:
                    code.get(currentProblemIndex)?.get(selectedLanguage) || "",
                },
              });
              setTestStatus({
                t: "Awaiting",
                c: null,
              });
              room.sendCommand({
                t: "TestCode",
                c: {
                  language: selectedLanguage,
                  testCases: problems[currentProblemIndex].defaultTestCases,
                },
              });
            })}
          >
            Test
          </Button>
          <Button
            w="28"
            isLoading={testStatus.t === "Awaiting"}
            onClick={throttle(() => {
              setTestStatus({
                t: "Awaiting",
                c: null,
              });
              room.sendCommand({
                t: "SubmitCode",
                c: {
                  problemIndex: currentProblemIndex,
                  language: selectedLanguage,
                },
              });
            })}
          >
            Submit
          </Button>
        </HStack>
      </Box>
    </Box>
  );
};
