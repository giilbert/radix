import { Problem, TestCase } from "@/types/problem";
import {
  Box,
  Code,
  Divider,
  HStack,
  Heading,
  Tag,
  Text,
} from "@chakra-ui/react";
import { MarkdownRender } from "../ui/markdown-render";
import { DifficultyTag } from "./difficulty-tag";

export const ProblemView: React.FC<{
  problem: Omit<Problem, "defaultTestCases"> & { testCases: TestCase[] };
}> = ({ problem }) => {
  return (
    <>
      <Heading mb="2">{problem.title}</Heading>
      <DifficultyTag difficulty={problem.difficulty} />
      <Divider my="2" />
      <MarkdownRender content={problem.description} />
      <Divider my="2" />
      <Heading mb="2" size="md" as="h2">
        Test Cases
      </Heading>
      {problem.testCases.map((testCase, i) => (
        <HStack key={i} mb="2">
          <Code>
            {testCase.input} =&gt; {testCase.output}
          </Code>
        </HStack>
      ))}
    </>
  );
};
