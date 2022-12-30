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

export const Editors: React.FC = () => {
  const { data: session } = useSession();
  const room = useRoom();
  const {
    users,
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

  const isResultsActive = testStatus.t !== "None";

  if (users.length === 0) return <Text>Loading</Text>;
  if (!session) return <Text>Loading</Text>;

  return (
    <Box>
      <Tabs
        h="100%"
        defaultIndex={users.findIndex((user) => user.id === session.user.id)}
      >
        <TabList h="3rem">
          {users.map((user) => (
            <Tab key={user.id}>{user.name}</Tab>
          ))}
        </TabList>

        <TabPanels h={`calc(100vh${isResultsActive ? " - 26rem" : " - 6rem"})`}>
          {users.map((user) => {
            return (
              <TabPanel p="0" h="100%" key={user.id}>
                {user.id !== session.user.id && (
                  <Text key={user.id}>No peeking</Text>
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
            onClick={throttle(() => {
              if (!problems) return;

              console.log(problems[currentProblemIndex].defaultTestCases);

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
