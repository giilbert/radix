import {
  Box,
  Code,
  Heading,
  HStack,
  Skeleton,
  Text,
  Textarea,
  VStack,
} from "@chakra-ui/react";
import { FiCheck, FiX } from "react-icons/fi";
import ReactConfetti from "react-confetti";
import { useRoomData } from "./room-provider";
import { useState } from "react";

export const TestResults: React.FC = () => {
  const testStatus = useRoomData((s) => s.testStatus);

  const confetti =
    testStatus.t === "Response" && testStatus.c.t === "AllTestsPassed";

  return (
    <Box
      w="full"
      h="20rem"
      bgColor="blackAlpha.400"
      p="2"
      overflow={testStatus.t === "Awaiting" ? "hidden" : "auto"}
    >
      {testStatus.t === "Awaiting" && (
        <Box>
          <Heading fontSize="1.4rem">Running...</Heading>
          <Skeleton w="full" h="24" mt="2" borderRadius="md" />
          <Skeleton w="full" h="24" mt="2" borderRadius="md" />
          <Skeleton w="full" h="24" mt="2" borderRadius="md" />
        </Box>
      )}
      {testStatus.t === "Response" && testStatus.c.t === "Ran" && (
        <>
          {testStatus.c.c.failedTests.length > 0 && (
            <Heading fontSize="1.4rem" mb="3">
              Failed
            </Heading>
          )}
          <VStack alignItems="start">
            {testStatus.c.c.failedTests.map((testCase, i) => (
              <Box
                key={i}
                border="solid 2px"
                borderColor="red.500"
                w="full"
                p="2"
                borderRadius="md"
              >
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
              border="solid 2px"
              borderColor="green.600"
              w="full"
              p="2"
              borderRadius="sm"
              mt="1"
            >
              <FiCheck />
              <Code bgColor="gray.800">
                {testCase.input} =&gt; {testCase.output}
              </Code>
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
          color="red.600"
          spellCheck="false"
          resize="none"
        />
      )}
      {testStatus.t === "Response" && testStatus.c.t === "AllTestsPassed" && (
        <>
          <Heading fontSize="1.4rem" mb="3">
            All tests passed in{" "}
            {testStatus.c.c.runtime === 0 ? "<1" : testStatus.c.c.runtime}ms!
          </Heading>

          <Text>Move on to the next question!</Text>
        </>
      )}

      <ReactConfetti
        width={window.innerWidth}
        height={window.innerHeight}
        numberOfPieces={confetti ? 500 : 0}
        recycle={false}
        onConfettiComplete={(confetti) => confetti?.reset()}
      />
    </Box>
  );
};
