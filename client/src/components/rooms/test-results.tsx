import {
  Box,
  Code,
  Heading,
  HStack,
  Text,
  Textarea,
  VStack,
} from "@chakra-ui/react";
import { useRoomData } from "./room-provider";

export const TestResults: React.FC = () => {
  const testStatus = useRoomData((s) => s.testStatus);

  return (
    <Box w="full" h="20rem" bgColor="blackAlpha.400" p="2">
      {testStatus.t === "Awaiting" && <Text>running ur stupid code...</Text>}
      {testStatus.t === "Response" && testStatus.c.t === "Ran" && (
        <>
          {testStatus.c.c.failedTests.length > 0 && (
            <Heading fontSize="1.4rem" mb="3">
              Failed
            </Heading>
          )}
          <VStack alignItems="start">
            {testStatus.c.c.failedTests.map((testCase, i) => (
              <Box key={i} bgColor="red.600" w="full" p="2" borderRadius="sm">
                <Code bgColor="gray.800">Input: {testCase.input}</Code> <br />
                <Code bgColor="gray.800">Expected: {testCase.expected}</Code>
                <br />
                <Code bgColor="gray.800">Got: {testCase.output}</Code>
              </Box>
            ))}
          </VStack>

          {testStatus.c.c.okayTests.length > 0 && (
            <>
              {testStatus.c.c.failedTests.length > 0 && (
                <hr style={{ margin: "1rem 0" }} />
              )}
              <Heading fontSize="1.4rem" mb="3">
                Passed
              </Heading>
            </>
          )}

          {testStatus.c.c.okayTests.map((testCase, i) => (
            <HStack
              key={i}
              bgColor="green.600"
              w="full"
              p="2"
              borderRadius="sm"
            >
              <Code bgColor="gray.800">{testCase.input}</Code>
            </HStack>
          ))}
        </>
      )}
      {testStatus.t === "Response" && testStatus.c.t === "Error" && (
        <Textarea
          defaultValue={testStatus.c.c.message}
          fontFamily="mono"
          h="full"
          contentEditable={false}
          color="whiteAlpha.900"
          spellCheck="false"
          resize="none"
        />
      )}
    </Box>
  );
};
