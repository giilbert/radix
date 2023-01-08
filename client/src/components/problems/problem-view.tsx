import { Problem } from "@/types/problem";
import { Heading, Tag, Text } from "@chakra-ui/react";
import { DifficultyTag } from "./difficulty-tag";

export const ProblemView: React.FC<{ problem: Problem }> = ({ problem }) => {
  return (
    <>
      <Heading mb="2">{problem.title}</Heading>
      <DifficultyTag difficulty={problem.difficulty} />

      <hr />

      <Text>{problem.description}</Text>
    </>
  );
};
