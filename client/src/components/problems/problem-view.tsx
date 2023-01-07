import { Problem } from "@/types/problem";
import { Heading, Text } from "@chakra-ui/react";

export const ProblemView: React.FC<{ problem: Problem }> = ({ problem }) => (
  <>
    <Heading>{problem.title}</Heading>
    <Text>{problem.description}</Text>
  </>
);
